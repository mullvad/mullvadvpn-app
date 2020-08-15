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

pub use self::imp::Error;

pub struct MonitorHandle(imp::MonitorHandle);

impl MonitorHandle {
    pub async fn is_offline(&mut self) -> bool {
        self.0.is_offline().await
    }
}

pub async fn spawn_monitor(
    sender: Weak<UnboundedSender<TunnelCommand>>,
    #[cfg(target_os = "android")] android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    Ok(MonitorHandle(
        imp::spawn_monitor(
            sender,
            #[cfg(target_os = "android")]
            android_context,
        )
        .await?,
    ))
}
