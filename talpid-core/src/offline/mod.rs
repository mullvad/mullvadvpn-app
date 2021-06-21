#[cfg(target_os = "linux")]
use crate::routing::RouteManagerHandle;
use crate::tunnel_state_machine::TunnelCommand;
use futures::channel::mpsc::UnboundedSender;
use std::sync::Weak;
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

lazy_static::lazy_static! {
    /// Disables offline monitor
    static ref FORCE_DISABLE_OFFLINE_MONITOR: bool = std::env::var("TALPID_DISABLE_OFFLINE_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false);
}

pub use self::imp::Error;

pub struct MonitorHandle(Option<imp::MonitorHandle>);

impl MonitorHandle {
    pub async fn is_offline(&mut self) -> bool {
        match self.0.as_mut() {
            Some(monitor) => monitor.is_offline().await,
            None => false,
        }
    }
}

pub async fn spawn_monitor(
    sender: Weak<UnboundedSender<TunnelCommand>>,
    #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
    #[cfg(target_os = "android")] android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    let monitor = if !*FORCE_DISABLE_OFFLINE_MONITOR {
        Some(
            imp::spawn_monitor(
                sender,
                #[cfg(target_os = "linux")]
                route_manager,
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
