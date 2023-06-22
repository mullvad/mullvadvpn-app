use futures::channel::mpsc::UnboundedSender;
use once_cell::sync::Lazy;
#[cfg(not(target_os = "android"))]
use talpid_routing::RouteManagerHandle;
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;

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
    pub async fn host_is_offline(&self) -> bool {
        match self.0.as_ref() {
            Some(monitor) => monitor.host_is_offline().await,
            None => false,
        }
    }
}

pub async fn spawn_monitor(
    sender: UnboundedSender<bool>,
    #[cfg(not(target_os = "android"))] route_manager: RouteManagerHandle,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
    #[cfg(target_os = "android")] android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    let monitor = if !*FORCE_DISABLE_OFFLINE_MONITOR {
        Some(
            imp::spawn_monitor(
                sender,
                #[cfg(not(target_os = "android"))]
                route_manager,
                #[cfg(target_os = "linux")]
                fwmark,
                #[cfg(target_os = "android")]
                android_context,
            )
            .await?,
        )
    } else {
        None
    };

    Ok(MonitorHandle(monitor))
}
