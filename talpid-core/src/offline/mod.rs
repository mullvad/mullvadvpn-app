#[cfg(target_os = "android")]
use crate::connectivity_listener::ConnectivityListener;
use futures::channel::mpsc::UnboundedSender;
use std::sync::LazyLock;
#[cfg(not(target_os = "android"))]
use talpid_routing::RouteManagerHandle;
use talpid_types::{ErrorExt, net::Connectivity};

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

/// Disables offline monitor
static FORCE_DISABLE_OFFLINE_MONITOR: LazyLock<bool> = LazyLock::new(|| {
    // HACK: It is fair to assume that a router has a stable connection and won't drop out very
    // often. An offline monitor might induce unwarranted reconnects, which would be very annoying.
    // TODO: Find a way to conditionally disable the offline monitor.
    true
});

pub struct MonitorHandle(Option<imp::MonitorHandle>);

impl MonitorHandle {
    pub async fn connectivity(&self) -> Connectivity {
        match self.0.as_ref() {
            Some(monitor) => monitor.connectivity().await,
            None => Connectivity::PresumeOnline,
        }
    }
}

pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    #[cfg(not(target_os = "android"))] route_manager: RouteManagerHandle,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
    #[cfg(target_os = "android")] connectivity_listener: ConnectivityListener,
) -> MonitorHandle {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        imp::spawn_monitor(
            sender,
            #[cfg(not(target_os = "android"))]
            route_manager,
            #[cfg(target_os = "linux")]
            fwmark,
            #[cfg(target_os = "android")]
            connectivity_listener,
        )
        .await
        .inspect_err(|error| {
            log::warn!(
                "{}",
                error.display_chain_with_msg("Failed to spawn offline monitor")
            );
        })
        .ok()
    };

    MonitorHandle(monitor)
}
