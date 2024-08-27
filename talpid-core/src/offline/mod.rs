use futures::channel::mpsc::UnboundedSender;
use std::sync::LazyLock;
#[cfg(not(target_os = "android"))]
use talpid_routing::RouteManagerHandle;
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;
use talpid_types::{net::Connectivity, ErrorExt};

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
    std::env::var("TALPID_DISABLE_OFFLINE_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false)
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
    #[cfg(target_os = "android")] android_context: AndroidContext,
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
            android_context,
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
