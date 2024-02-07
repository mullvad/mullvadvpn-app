use futures::channel::mpsc::UnboundedSender;
use once_cell::sync::Lazy;
#[cfg(not(target_os = "android"))]
use talpid_routing::RouteManagerHandle;
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;
use talpid_types::net::Connectivity;

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
static FORCE_DISABLE_OFFLINE_MONITOR: Lazy<bool> = Lazy::new(|| {
    std::env::var("TALPID_DISABLE_OFFLINE_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false)
});

pub use self::imp::Error;

pub struct MonitorHandle(Option<imp::MonitorHandle>);

impl MonitorHandle {
    pub async fn connectivity(&self) -> Connectivity {
        match self.0.as_ref() {
            Some(monitor) => monitor.connectivity().await,
            None => Connectivity::PresumeOnline,
        }
    }
}

#[cfg(not(target_os = "android"))]
pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    route_manager: RouteManagerHandle,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Result<MonitorHandle, Error> {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        Some(
            imp::spawn_monitor(
                sender,
                route_manager,
                #[cfg(target_os = "linux")]
                fwmark,
            )
            .await?,
        )
    };

    Ok(MonitorHandle(monitor))
}

#[cfg(target_os = "android")]
pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        Some(imp::spawn_monitor(sender, android_context).await?)
    };

    Ok(MonitorHandle(monitor))
}
