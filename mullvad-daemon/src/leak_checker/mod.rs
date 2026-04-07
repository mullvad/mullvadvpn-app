use futures::{FutureExt, future::pending, select};
pub use mullvad_leak_checker::LeakInfo;
use std::time::Duration;
use talpid_routing::RouteManagerHandle;
use talpid_types::{
    net::{Endpoint, TunnelEndpoint},
    tunnel::TunnelStateTransition,
};
use tokio::{sync::mpsc, task::JoinHandle};

/// An actor that tries to leak traffic outside the tunnel while we are connected.
pub struct LeakChecker {
    task_event_tx: mpsc::UnboundedSender<TaskEvent>,
}

/// [LeakChecker] internal task state.
struct Task {
    events_rx: mpsc::UnboundedReceiver<TaskEvent>,
    route_manager: RouteManagerHandle,
    callbacks: Vec<Box<dyn LeakCheckerCallback>>,
    leak_test: Option<JoinHandle<Option<LeakInfo>>>,
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
            leak_test: None,
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
            let leak_test = async {
                match &mut self.leak_test {
                    Some(task) => task.await,
                    None => pending().await,
                }
            };
            select! {
                event = self.events_rx.recv().fuse() => {
                    let Some(event) = event else {
                        break; // All LeakChecker handles dropped.
                    };

                    match event {
                        TaskEvent::NewTunnelState(s) => self.on_new_tunnel_state(s),
                        TaskEvent::AddCallback(c) => self.on_add_callback(c),
                    }
                }
                leak_info = leak_test.fuse() => {
                    self.leak_test = None;
                    if let Ok(Some(leak_info)) = leak_info {
                        self.on_leak_test_complete(leak_info);
                    }
                }
            }
        }
    }

    fn on_add_callback(&mut self, c: Box<dyn LeakCheckerCallback>) {
        self.callbacks.push(c);
    }

    fn on_new_tunnel_state(&mut self, tunnel_state: TunnelStateTransition) {
        // If the tunnel state changed, we need to abort any in-progress leak test,
        // since our results might become invalid.
        if let Some(task) = self.leak_test.take() {
            task.abort();
        }

        // Only run leak tests if we are connected.                                                  ..
        if let TunnelStateTransition::Connected(tunnel) = tunnel_state {
            self.leak_test = Some(self.spawn_leak_test(tunnel));
        };
    }

    fn on_leak_test_complete(&mut self, leak_info: LeakInfo) {
        log::debug!("Leak detected: {leak_info:?}");

        self.callbacks
            .retain_mut(|callback| callback.on_leak(leak_info.clone()) == CallbackResult::Ok);
    }

    fn spawn_leak_test(&self, tunnel: TunnelEndpoint) -> JoinHandle<Option<LeakInfo>> {
        let ping_destination = tunnel.endpoint;
        let route_manager = self.route_manager.clone();

        tokio::spawn(async move {
            // Give the connection a lttle time to settle before starting the test.
            tokio::time::sleep(Duration::from_secs(5)).await;
            let leak_result = check_for_leaks(&route_manager, ping_destination).await;

            let leak_info = match leak_result {
                Ok(Some(leak_info)) => leak_info,
                Ok(None) => {
                    log::debug!("No leak detected");
                    return None;
                }
                Err(e) => {
                    log::debug!("Leak check errored: {e:#?}");
                    return None;
                }
            };

            Some(leak_info)
        })
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
