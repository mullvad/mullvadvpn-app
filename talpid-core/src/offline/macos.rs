use futures::{channel::mpsc::UnboundedSender, Future, StreamExt};
use std::sync::{Arc, Weak};
use talpid_types::ErrorExt;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize route monitor")]
    StartMonitorError(#[error(source)] crate::routing::PlatformError),
}

pub struct MonitorHandle {
    _notify_tx: Arc<UnboundedSender<bool>>,
}

impl MonitorHandle {
    /// Host is considered to be offline if the IPv4 internet is considered to be unreachable by the
    /// given reachability flags *or* there are no active physical interfaces.
    pub async fn is_offline(&self) -> bool {
        !exists_non_tunnel_default_route().await
    }
}

async fn exists_non_tunnel_default_route() -> bool {
    match crate::routing::get_default_routes().await {
        Ok((Some(node), _)) | Ok((None, Some(node))) => {
            let route_exists = node
                .get_device()
                .map(|iface_name| !iface_name.contains("tun"))
                .unwrap_or(true);
            log::debug!("Assuming non-tunnel default route exists due to {:?}", node);
            route_exists
        }
        Ok((None, None)) => {
            log::debug!("No default routes exist, assuming machine is offline");
            false
        }
        Err(err) => {
            log::error!(
                "{}",
                err.display_chain_with_msg(
                    "Failed to obtain default routes, assuming machine is online."
                )
            );
            true
        }
    }
}
pub async fn spawn_monitor(notify_tx: UnboundedSender<bool>) -> Result<MonitorHandle, Error> {
    let notify_tx = Arc::new(notify_tx);

    let context = OfflineStateContext {
        sender: Arc::downgrade(&notify_tx),
        is_offline: !exists_non_tunnel_default_route().await,
    };

    let route_monitor = watch_route_monitor(context)?;
    tokio::spawn(route_monitor);
    Ok(MonitorHandle {
        _notify_tx: notify_tx,
    })
}

fn watch_route_monitor(
    mut context: OfflineStateContext,
) -> Result<impl Future<Output = ()>, Error> {
    let mut monitor = crate::routing::listen_for_default_route_changes()?;

    Ok(async move {
        while let Some(_route_change) = monitor.next().await {
            context.new_state(!exists_non_tunnel_default_route().await);
            if context.should_shut_down() {
                break;
            }
        }
        log::debug!("Stopping offline monitor");
    })
}

#[derive(Clone)]
struct OfflineStateContext {
    sender: Weak<UnboundedSender<bool>>,
    is_offline: bool,
}

impl OfflineStateContext {
    fn should_shut_down(&self) -> bool {
        self.sender.upgrade().is_none()
    }

    fn new_state(&mut self, is_offline: bool) {
        if self.is_offline != is_offline {
            self.is_offline = is_offline;
            if let Some(sender) = self.sender.upgrade() {
                let _ = sender.unbounded_send(is_offline);
            }
        }
    }
}
