use crate::tunnel_state_machine::TunnelCommand;
use futures::{
    sync::mpsc::{self, UnboundedSender},
    Future, Sink,
};
use tokio_core::reactor::Handle;

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

pub mod graceful_stream;

pub use self::imp::{is_offline, Error};

pub struct MonitorHandle(imp::MonitorHandle);

const GRACE_PERIOD: u64 = 3;

pub fn spawn_monitor(
    sender: UnboundedSender<TunnelCommand>,
    handle: &Handle,
) -> Result<MonitorHandle, Error> {
    let (tx, rx) = mpsc::unbounded();
    let damper =
        graceful_stream::GracefulStream::new(rx, ::std::time::Duration::from_secs(GRACE_PERIOD));
    handle.spawn(
        sender
            .sink_map_err(|_e| log::trace!("Tunnel command receiver dropped"))
            .send_all(damper)
            .map(|_| ()),
    );

    #[cfg(not(target_os = "linux"))]
    {
        return Ok(MonitorHandle(imp::spawn_monitor(tx)?));
    }
    #[cfg(target_os = "linux")]
    {
        return Ok(MonitorHandle(imp::spawn_monitor(tx, handle)?));
    }
}
