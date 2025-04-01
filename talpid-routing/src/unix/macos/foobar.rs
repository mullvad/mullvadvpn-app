use std::{collections::HashMap, future::pending, time::Duration};

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    select_biased, FutureExt, StreamExt,
};
use tokio::{
    runtime,
    time::{sleep_until, Instant},
};

use crate::imp::imp::interface::NetworkServiceDetails;

use super::{
    interface::{Family, InterfaceEvent, PrimaryInterfaceDetails, PrimaryInterfaceMonitor},
    ip_map::IpMap,
    DefaultRoute,
};

const NO_ROUTE_GRACE_TIME: Duration = Duration::from_secs(5);

pub fn start_listener(
    monitor: PrimaryInterfaceMonitor,
    event_rx: UnboundedReceiver<Vec<InterfaceEvent>>,
) -> (
    UnboundedReceiver<Option<DefaultRoute>>,
    UnboundedReceiver<Option<DefaultRoute>>,
) {
    let (route_v4_tx, route_v4_rx) = mpsc::unbounded();
    let (route_v6_tx, route_v6_rx) = mpsc::unbounded();

    let mut route_tx = IpMap::new();
    route_tx.insert(Family::V4, route_v4_tx);
    route_tx.insert(Family::V6, route_v6_tx);

    let monitor = DefaultRouteMonitor {
        monitor,
        event_rx,
        route_tx,
        current_route: IpMap::new(),
        primary_interfaces: IpMap::new(),
    };

    tokio::task::spawn_blocking(move || {
        runtime::Handle::current().block_on(monitor.run());
    });

    let route_v4_rx = delay_no_route_events(route_v4_rx);
    let route_v6_rx = delay_no_route_events(route_v6_rx);

    (route_v4_rx, route_v6_rx)
}

/// Delay `None`-events by [NO_ROUTE_GRACE_TIME].
///
/// When receiving a `None` on the channel, a timer will start. If no `Some`s are received within
/// the deadline, a `None` will be sent.
///
/// Some `None`s may be dropped, but `Some`-values are passed along immediately.
fn delay_no_route_events(
    mut fast_rx: UnboundedReceiver<Option<DefaultRoute>>,
) -> UnboundedReceiver<Option<DefaultRoute>> {
    let (slow_tx, slow_rx) = mpsc::unbounded();

    tokio::task::spawn(async move {
        let mut no_route_grace_timeout = None;

        loop {
            let no_route_grace_timer = async {
                match no_route_grace_timeout {
                    None => pending().await,
                    Some(time) => sleep_until(time).await,
                };
            };

            select_biased! {
                route = fast_rx.next() => {
                    let Some(route) = route else { return };

                    if route.is_some() {
                        no_route_grace_timeout = None;
                        if slow_tx.unbounded_send(route).is_err() {
                            return;
                        };

                    } else if no_route_grace_timeout.is_none() {
                        // FIXME: remove this log
                        log::debug!("New route is None, starting grace timer.");
                        no_route_grace_timeout = Some(Instant::now() + NO_ROUTE_GRACE_TIME);
                    }
                }

                _ = no_route_grace_timer.fuse() => {
                    no_route_grace_timeout = None;
                    if slow_tx.unbounded_send(None).is_err() {
                        return;
                    };
                }
            }
        }
    });

    slow_rx
}

struct DefaultRouteMonitor {
    monitor: PrimaryInterfaceMonitor,
    event_rx: UnboundedReceiver<Vec<InterfaceEvent>>,

    route_tx: IpMap<UnboundedSender<Option<DefaultRoute>>>,

    current_route: IpMap<DefaultRoute>,
    primary_interfaces: IpMap<PrimaryInterfaceDetails>,
}

impl DefaultRouteMonitor {
    async fn run(mut self) {
        for family in [Family::V4, Family::V6] {
            let route = self.monitor.get_route(family);

            self.current_route.set(family, route.clone());
            if let Some(tx) = self.route_tx.get(family) {
                let _ = tx.unbounded_send(route);
            }
        }

        while let Some(events) = self.event_rx.next().await {
            if self.route_tx.is_empty() {
                break;
            }

            self.handle_events(events);
        }
    }

    fn handle_events(&mut self, events: Vec<InterfaceEvent>) {
        // Split events by address family and handle them seperately.
        let mut ipv4_events = vec![];
        let mut ipv6_events = vec![];
        for event in events {
            match event.family() {
                Family::V4 => ipv4_events.push(event),
                Family::V6 => ipv6_events.push(event),
            }
        }

        self.handle_events_for_family(Family::V4, ipv4_events);
        self.handle_events_for_family(Family::V6, ipv6_events);
    }

    fn handle_events_for_family(&mut self, family: Family, events: Vec<InterfaceEvent>) {
        enum Change<T> {
            New(T),
            Removed,
        }

        // Go through the events and figure out if the primary interface changed.
        let mut primary_interface_change: Option<Change<PrimaryInterfaceDetails>> = None;
        for event in &events {
            let InterfaceEvent::PrimaryInterfaceUpdate { new_value, .. } = event else {
                continue;
            };

            primary_interface_change = Some(match new_value {
                Some(new_value) => Change::New(new_value.clone()),
                None => Change::Removed,
            });
        }

        // Collect all NetworkServiceUpdates into a HashMap.
        let changed_services: HashMap<String, Change<NetworkServiceDetails>> = events
            .into_iter()
            .filter_map(|service| {
                let InterfaceEvent::NetworkServiceUpdate {
                    service_id,
                    new_value,
                    ..
                } = service
                else {
                    return None;
                };

                let change = match new_value {
                    Some(service) => Change::New(service),
                    None => Change::Removed,
                };

                Some((service_id, change))
            })
            .collect();

        // Figure out if anything interesting happened.
        // Things we care about:
        //  - The primary interface changed.
        //  - The service of the primary interface changed.
        //  - If we're NOT using the primary interface, we care about whether ANY service changed.
        let an_important_service_changed =
            if let Some(primary_interface) = self.primary_interfaces.get(family) {
                changed_services.contains_key(&primary_interface.service_id)
            } else {
                !changed_services.is_empty()
            };

        // If nothing interesting has happened, just return.
        if primary_interface_change.is_none() && !an_important_service_changed {
            return;
        }

        // Figure out what the new default route should be.
        // Match on the new primary interface, and the previous primary interface
        let new_route = match (
            primary_interface_change.as_ref(),
            self.primary_interfaces.get(family),
        ) {
            // This match covers two cases:
            // - The primary interface changed.
            // - The primary interface didn't change, and we have one from before.
            (Some(Change::New(interface)), _) | (None, Some(interface)) => changed_services
                .get(&interface.service_id)
                .and_then(|change| match change {
                    Change::New(service) => Some(service),
                    Change::Removed => None,
                })
                .and_then(|service| self.monitor.route_from_service(service))
                .or_else(|| self.monitor.get_route_by_service_order(family)),

            // This match covers the case where the primary interface was removed, or it never
            // existed. In this case we iterate over all network services and pick the first good
            // one.
            _ => self.monitor.get_route_by_service_order(family),
        };

        // If the new route is the same as the old one, do nohing.
        if new_route.as_ref() == self.current_route.get(family) {
            return;
        }

        self.current_route.set(family, new_route.clone());
        if let Some(tx) = self.route_tx.get(family) {
            if tx.unbounded_send(new_route).is_err() {
                self.route_tx.remove(family);
            }
        }
    }
}
