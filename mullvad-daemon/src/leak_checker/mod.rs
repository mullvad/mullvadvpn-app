use futures::{FutureExt, select};
pub use mullvad_leak_checker::LeakInfo;
use std::time::Duration;
use talpid_routing::RouteManagerHandle;
use talpid_types::{net::Endpoint, tunnel::TunnelStateTransition};
use tokio::sync::mpsc;

/// An actor that tries to leak traffic outside the tunnel while we are connected.
pub struct LeakChecker {
    task_event_tx: mpsc::UnboundedSender<TaskEvent>,
}

/// [LeakChecker] internal task state.
struct Task {
    events_rx: mpsc::UnboundedReceiver<TaskEvent>,
    route_manager: RouteManagerHandle,
    callbacks: Vec<Box<dyn LeakCheckerCallback>>,
}

enum TaskEvent {
    NewTunnelState(TunnelStateTransition),
    AddCallback(Box<dyn LeakCheckerCallback>),
}

#[derive(PartialEq, Eq)]
pub enum CallbackResult {
    /// Callback completed successfully
    Ok,

    /// Callback is no longer valid and should be dropped.
    Drop,
}

pub trait LeakCheckerCallback: Send + 'static {
    fn on_leak(&mut self, info: LeakInfo) -> CallbackResult;
}

impl LeakChecker {
    pub fn new(route_manager: RouteManagerHandle) -> Self {
        let (task_event_tx, events_rx) = mpsc::unbounded_channel();

        let task = Task {
            events_rx,
            route_manager,
            callbacks: vec![],
        };

        tokio::task::spawn(task.run());

        LeakChecker { task_event_tx }
    }

    /// Call when we transition to a new tunnel state.
    pub fn on_tunnel_state_transition(&mut self, tunnel_state: TunnelStateTransition) {
        self.send(TaskEvent::NewTunnelState(tunnel_state))
    }

    /// Call `callback` if a leak is detected.
    pub fn add_leak_callback(&mut self, callback: impl LeakCheckerCallback) {
        self.send(TaskEvent::AddCallback(Box::new(callback)))
    }

    /// Send a [TaskEvent] to the running [Task];
    fn send(&mut self, event: TaskEvent) {
        if self.task_event_tx.send(event).is_err() {
            panic!("LeakChecker unexpectedly closed");
        }
    }
}

impl Task {
    async fn run(mut self) {
        loop {
            let Some(event) = self.events_rx.recv().await else {
                break; // All LeakChecker handles dropped.
            };

            match event {
                TaskEvent::NewTunnelState(s) => self.on_new_tunnel_state(s).await,
                TaskEvent::AddCallback(c) => self.on_add_callback(c),
            }
        }
    }

    fn on_add_callback(&mut self, c: Box<dyn LeakCheckerCallback>) {
        self.callbacks.push(c);
    }

    async fn on_new_tunnel_state(&mut self, mut tunnel_state: TunnelStateTransition) {
        'leak_test: loop {
            let TunnelStateTransition::Connected(tunnel) = &tunnel_state else {
                break 'leak_test;
            };

            let ping_destination = tunnel.endpoint;
            let route_manager = self.route_manager.clone();
            let leak_test = async {
                // Give the connection a little time to settle before starting the test.
                tokio::time::sleep(Duration::from_millis(5000)).await;

                check_for_leaks(&route_manager, ping_destination).await
            };

            // Make sure the tunnel state doesn't change while we're doing the leak test.
            // If that happens, then our results might be invalid.
            let another_tunnel_state = async {
                'listen_for_events: while let Some(event) = self.events_rx.recv().await {
                    let new_state = match event {
                        TaskEvent::NewTunnelState(tunnel_state) => tunnel_state,
                        TaskEvent::AddCallback(c) => {
                            self.on_add_callback(c);
                            continue 'listen_for_events;
                        }
                    };

                    if let TunnelStateTransition::Connected(..) = new_state {
                        // Still connected, all is well...
                    } else {
                        // Tunnel state changed! We have to discard the leak test and try again.
                        tunnel_state = new_state;
                        break 'listen_for_events;
                    }
                }
            };

            let leak_result = select! {
                // If tunnel state changes, restart the test.
                _ = another_tunnel_state.fuse() => continue 'leak_test,

                leak_result = leak_test.fuse() => leak_result,
            };

            let leak_info = match leak_result {
                Ok(Some(leak_info)) => leak_info,
                Ok(None) => {
                    log::debug!("No leak detected");
                    break 'leak_test;
                }
                Err(e) => {
                    log::debug!("Leak check errored: {e:#?}");
                    break 'leak_test;
                }
            };

            log::debug!("Leak detected: {leak_info:?}");

            self.callbacks
                .retain_mut(|callback| callback.on_leak(leak_info.clone()) == CallbackResult::Ok);

            break 'leak_test;
        }
    }
}

#[cfg(target_os = "android")]
#[expect(clippy::unused_async)]
async fn check_for_leaks(
    _route_manager: &RouteManagerHandle,
    _destination: Endpoint,
) -> anyhow::Result<Option<LeakInfo>> {
    // TODO: We currently don't have a way to get the non-tunnel interface on Android.
    Ok(None)
}

#[cfg(not(target_os = "android"))]
async fn check_for_leaks(
    route_manager: &RouteManagerHandle,
    destination: Endpoint,
) -> anyhow::Result<Option<LeakInfo>> {
    use anyhow::{Context, anyhow};
    use mullvad_leak_checker::{LeakStatus, traceroute::TracerouteOpt};

    #[cfg(target_os = "linux")]
    let interface = {
        // By setting FWMARK, we are effectively getting the same route as when using split tunneling.
        let route = route_manager
            .get_destination_route(destination.address.ip(), Some(mullvad_types::TUNNEL_FWMARK))
            .await
            .context("Failed to get route to relay")?
            .ok_or(anyhow!("No route to relay"))?;

        route
            .get_node()
            .get_device()
            .context("No device for default route")?
            .to_string()
            .into()
    };

    #[cfg(target_os = "macos")]
    let interface = {
        let (v4_route, v6_route) = route_manager
            .get_default_routes()
            .await
            .context("Failed to get default interface")?;
        let index = if destination.address.is_ipv4() {
            let v4_route = v4_route.context("Missing IPv4 default interface")?;
            v4_route.interface_index
        } else {
            let v6_route = v6_route.context("Missing IPv6 default interface")?;
            v6_route.interface_index
        };

        let index =
            std::num::NonZeroU32::try_from(u32::from(index)).context("Interface index was 0")?;
        mullvad_leak_checker::Interface::Index(index)
    };

    #[cfg(target_os = "windows")]
    let interface = {
        use std::net::IpAddr;
        use talpid_windows::net::AddressFamily;

        let _ = route_manager; // don't need this on windows

        let family = match destination.address.ip() {
            IpAddr::V4(..) => AddressFamily::Ipv4,
            IpAddr::V6(..) => AddressFamily::Ipv6,
        };

        let route = talpid_routing::get_best_default_route(family)
            .context("Failed to get best default route")?
            .ok_or_else(|| anyhow!("No default route found"))?;

        mullvad_leak_checker::Interface::Luid(route.iface)
    };

    log::debug!("Attempting to leak traffic on interface {interface:?} to {destination}");

    mullvad_leak_checker::traceroute::try_run_leak_test(&TracerouteOpt {
        interface,
        destination: destination.address.ip(),

        #[cfg(unix)]
        port: None,
        #[cfg(unix)]
        exclude_port: None,
        #[cfg(unix)]
        icmp: true,
    })
    .await
    .map_err(|e| anyhow!("{e:#}"))
    .map(|status| match status {
        LeakStatus::NoLeak => None,
        LeakStatus::LeakDetected(info) => Some(info),
    })
}

impl<T> LeakCheckerCallback for T
where
    T: FnMut(LeakInfo) -> bool + Send + 'static,
{
    fn on_leak(&mut self, info: LeakInfo) -> CallbackResult {
        if self(info) {
            CallbackResult::Ok
        } else {
            CallbackResult::Drop
        }
    }
}
