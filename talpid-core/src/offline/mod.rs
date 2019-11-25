use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;
use std::sync::Weak;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
#[path = "dummy.rs"]
mod imp;

pub use self::imp::Error;

pub struct MonitorHandle(imp::MonitorHandle);

impl MonitorHandle {
    pub fn is_offline(&self) -> bool {
        self.0.is_offline()
    }
}

pub fn spawn_monitor(sender: Weak<UnboundedSender<TunnelCommand>>) -> Result<MonitorHandle, Error> {
    Ok(MonitorHandle(imp::spawn_monitor(sender)?))
}
