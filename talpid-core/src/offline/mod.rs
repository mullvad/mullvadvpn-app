use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;

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

pub use self::imp::is_offline;

pub struct MonitorHandle(imp::MonitorHandle);

pub fn spawn_monitor(sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle, imp::Error> {
    Ok(MonitorHandle(imp::spawn_monitor(sender)?))
}
