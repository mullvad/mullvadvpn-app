use anyhow::anyhow;
use futures::{select, FutureExt};
use leak_checker::traceroute::TracerouteOpt;
pub use leak_checker::LeakInfo;
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
        // TODO: remove this if the above compiles on macos and android
        //tokio::task::spawn_blocking(|| Handle::current().block_on(task.run()));

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
                // TODO: is this necessary? is there some better way?
                // TODO: ether remove this or add some concrete motivation.
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

            for callback in &mut self.callbacks {
                callback.on_leak(leak_info.clone());
            }

            break 'leak_test;
        }
    }
}

async fn check_for_leaks(
    route_manager: &RouteManagerHandle,
    destination: Endpoint,
) -> anyhow::Result<Option<LeakInfo>> {
    // TODO (linux):
    // Use get_destination_route(ip, Some(fwmark)) to figure out default interface.
    // where ip is some unused example public ip, or maybe the relay ip
    #[cfg(target_os = "linux")]
    let interface = {
        let Ok(Some(route)) = route_manager
            .get_destination_route(destination.address.ip(), Some(mullvad_types::TUNNEL_FWMARK))
            .await
        else {
            todo!("no route to relay?");
        };

        route
            .get_node()
            .get_device()
            .expect("no device for default route??")
            .to_string()
            .into()
    };

    // TODO (android):
    // Maybe connectivity monitor?
    // It should be possible somehow. `ifconfig` can print interfaces.
    // needs further investigation
    #[cfg(target_os = "android")]
    let interface = todo!("get default interface");

    // TODO (macos):
    // get_default_route in route manager
    #[cfg(target_os = "macos")]
    let interface = todo!("get default interface");

    // TODO (windows):
    // Use default route monitor thingy. It should contain interfaces.
    // Can maybe use callback to subscribe for updates
    // get_best_route
    #[cfg(target_os = "windows")]
    let interface = {
        use std::net::IpAddr;
        use talpid_windows::net::AddressFamily;

        let family = match destination.address.ip() {
            IpAddr::V4(..) => AddressFamily::Ipv4,
            IpAddr::V6(..) => AddressFamily::Ipv6,
        };

        let Ok(Some(route)) = talpid_routing::get_best_default_route(family) else {
            todo!("no best default route");
        };

        leak_checker::Interface::Luid(route.iface)
    };

    log::debug!("attempting to leak traffic on interface {interface:?} to {destination}");

    // TODO: use UDP on windows
    leak_checker::traceroute::try_run_leak_test(&TracerouteOpt {
        interface,
        destination: destination.address.ip(),
        port: None,

        exclude_port: cfg!(target_os = "windows").then_some(destination.address.port()),
        icmp: cfg!(not(target_os = "windows")),
    })
    .await
    .map_err(|e| anyhow!("{e:#}"))
    .map(|status| match status {
        leak_checker::LeakStatus::NoLeak => None,
        leak_checker::LeakStatus::LeakDetected(info) => Some(info),
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
