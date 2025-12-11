use crate::connectivity_listener::{ConnectivityListener, Error};
use futures::channel::mpsc::UnboundedSender;
use talpid_types::net::Connectivity;

pub struct MonitorHandle {
    connectivity_listener: ConnectivityListener,
}

impl MonitorHandle {
    fn new(connectivity_listener: ConnectivityListener) -> Self {
        MonitorHandle {
            connectivity_listener,
        }
    }

    #[expect(clippy::unused_async)]
    pub async fn connectivity(&self) -> Connectivity {
        self.connectivity_listener.connectivity()
    }
}

#[expect(clippy::unused_async)]
pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    connectivity_listener: ConnectivityListener,
) -> Result<MonitorHandle, Error> {
    let mut monitor_handle = MonitorHandle::new(connectivity_listener);
    monitor_handle
        .connectivity_listener
        .set_connectivity_listener(sender);
    Ok(monitor_handle)
}
