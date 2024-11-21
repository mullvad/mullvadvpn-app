use anyhow::anyhow;
use leak_checker::traceroute::TracerouteOpt;
pub use leak_checker::LeakInfo;
use std::net::IpAddr;
use std::ops::ControlFlow;
use talpid_types::tunnel::TunnelStateTransition;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

/// An actor that tries to leak traffic outside the tunnel while we are connected.
pub struct LeakChecker {
    task_event_tx: mpsc::UnboundedSender<TaskEvent>,
}

/// [LeakChecker] internal task state.
struct Task {
    events_rx: mpsc::UnboundedReceiver<TaskEvent>,
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
    pub fn new() -> Self {
        let (task_event_tx, events_rx) = mpsc::unbounded_channel();

        let task = Task {
            events_rx,
            callbacks: vec![],
        };

        // TODO: fix task.run() not being Send
        //tokio::task::spawn(task.run());
        tokio::task::spawn_blocking(|| Handle::current().block_on(task.run()));

        LeakChecker { task_event_tx }
    }

    /// Call when we transition to a new tunnel state.
    pub fn on_tunnel_state_transition(&mut self, tunnel_state: TunnelStateTransition) {
        self.send(TaskEvent::NewTunnelState(tunnel_state))
    }

    pub fn add_leak_callback(&mut self, callback: impl LeakCheckerCallback) {
        self.send(TaskEvent::AddCallback(Box::new(callback)))
    }

    fn send(&mut self, event: TaskEvent) {
        if self.task_event_tx.send(event).is_err() {
            panic!("LeakChecker unexpectedly closed");
        }
    }

    ///// Wait until the leak detector detects a leak.
    /////
    ///// Ideally, this should never return.
    //pub async fn wait_for_leak(&self) -> LeakInfo {
    //    self.leak_rx
    //        .recv()
    //        .await
    //        .expect("LeakChecker unexpectedly closed")
    //}
}

impl Task {
    async fn run(mut self) {
        loop {
            let Some(event) = self.events_rx.recv().await else {
                break; // All LeakChecker handles dropped.
            };

            match event {
                TaskEvent::NewTunnelState(s) => {
                    if self.on_new_tunnel_state(s).await.is_break() {
                        break;
                    }
                }
                TaskEvent::AddCallback(c) => self.on_add_callback(c),
            }
        }
    }

    fn on_add_callback(&mut self, c: Box<dyn LeakCheckerCallback>) {
        self.callbacks.push(c);
    }

    async fn on_new_tunnel_state(
        &mut self,
        mut tunnel_state: TunnelStateTransition,
    ) -> ControlFlow<()> {
        'leak_test: loop {
            //let TunnelStateTransition::Connected(tunnel) = &tunnel_state else {
            let TunnelStateTransition::Connected(tunnel) = &tunnel_state else {
                return ControlFlow::Continue(());
            };

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            let ping_destination = tunnel.endpoint.address.ip();
            //let ping_destination = Ipv4Addr::new(185, 213, 154, 218);

            let interface = "wlan0"; // TODO

            let leak_info = match check_for_leaks(interface, ping_destination).await {
                Ok(Some(leak_info)) => leak_info,
                Ok(None) => {
                    log::debug!("No leak detected");
                    continue;
                }
                Err(e) => {
                    log::debug!("Leak check errored: {e:#?}");
                    return ControlFlow::Continue(());
                }
            };

            log::debug!("leak detected: {leak_info:?}");

            // Make sure the tunnel state didn't change while we were doing the leak test.
            // If that happened, then our results might be invalid.
            while let Ok(event) = self.events_rx.try_recv() {
                let new_state = match event {
                    TaskEvent::NewTunnelState(tunnel_state) => tunnel_state,
                    TaskEvent::AddCallback(c) => {
                        self.on_add_callback(c);
                        continue;
                    }
                };

                if let TunnelStateTransition::Connected(..) = new_state {
                    // Still connected, all is well...
                } else {
                    // Tunnel state changed! We have to discard the leak test and try again.
                    tunnel_state = new_state;
                    continue 'leak_test;
                }
            }

            for callback in &mut self.callbacks {
                callback.on_leak(leak_info.clone());
            }
            return ControlFlow::Continue(());
        }
    }
}

async fn check_for_leaks(interface: &str, destination: IpAddr) -> anyhow::Result<Option<LeakInfo>> {
    leak_checker::traceroute::try_run_leak_test(&TracerouteOpt {
        interface: interface.to_string(),
        destination,
        exclude_port: None,
        port: None,
        icmp: true,
    })
    .await
    .map_err(|e| anyhow!("{e:#}"))
    .map(|status| match status {
        leak_checker::LeakStatus::NoLeak => None,
        leak_checker::LeakStatus::LeakDetected(info) => Some(info),
    })
}

// async fn check_for_leaks(interface: &str, destination: IpAddr) -> anyhow::Result<Option<LeakInfo>> {
//     use std::mem::ManuallyDrop;
//     use std::os::fd::FromRawFd;
//     let client = surge_ping::Client::new(&surge_ping::Config {
//         sock_type_hint: socket2::Type::DGRAM,
//         kind: surge_ping::ICMP::V4,
//
//         // On desktop linux, we can bind directly to the interface.
//         interface: cfg!(target_os = "linux").then(|| interface.to_string()),
//
//         // On other systems, we resord to binding to the interfaces IP address instead.
//         bind: cfg!(not(target_os = "linux")).then(|| get_interface_ip(interface)),
//
//         ttl: None,
//         fib: None,
//     })
//     .context("Failed to create ping client")?;
//
//     // TODO: additional configuration?
//     let socket = client.get_socket();
//
//     // SAFETY: socket.get_native_sock returns an open fd.
//     // The socket2 socket is not used after we drop the client.
//     // We wrap the socket2 socket in a ManuallyDrop to prevent it from dropping the socket.
//     let socket = unsafe { socket2::Socket::from_raw_fd(socket.get_native_sock()) };
//     let socket = ManuallyDrop::new(socket);
//     let mut pinger = client.pinger(destination, PingIdentifier(12345)).await;
//
//     for ttl in 1..=5u16 {
//         let ping_seq = ttl;
//
//         socket
//             .set_ttl(u32::from(ttl))
//             .context("Failed to set TTL")?;
//
//         let (reply, _duration) = pinger
//             .ping(PingSequence(ping_seq), b"ABCDEFGHIJKLMNOP")
//             .await
//             .context("Failed to send ping")?;
//
//         println!("icmp_reply: {reply:?}");
//     }
//
//     todo!()
// }

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
