//! This module has been reimplemented multiple times, often to no avail, with main issues being
//! that the app gets stuck in an offline state, blocking all internet access and preventing the
//! user from connecting to a relay.
//!
//! Currently, this functionality is implemented by using `route monitor -n` to observe routing
//! table changes and then use the CLI once more to query if there exists a default route.
//! Generally, it is assumed that a machine is online if there exists a route to a public IP
//! address that isn't using a tunnel adapter. On macOS, there were various ways of deducing this:
//! - watching the `State:/Network/Global/IPv4`  key in SystemConfiguration via
//!  `system-configuration-rs`, relying on a CoreFoundation runloop to drive callbacks.
//!   The issue with this is that sometimes during early boot or after a re-install, the callbacks
//!   won't be called, often leaving the daemon stuck in an offline state.
//! - setting a callback via [`SCNetworkReachability`]. The callback should be called whenever the
//!   reachability of a remote host changes, but sometimes the callbacks just don't get called.
//! - [`NWPathMonitor`] is a macOS native interface to watch changes in the routing table. It works
//!   great, but it seems to deliver updates before they actually get added to the routing table,
//!   effectively calling our callbacks with routes that aren't yet usable, so starting tunnels
//!   would fail anyway. This would be the API to use if we were able to bind the sockets our tunnel
//!   implementations would use, but that is far too much complexity.
//!
//! [`SCNetworkReachability`]: https://developer.apple.com/documentation/systemconfiguration/scnetworkreachability-g7d
//! [`NWPathMonitor`]: https://developer.apple.com/documentation/network/nwpathmonitor
use futures::{channel::mpsc::UnboundedSender, Future, StreamExt};
use std::sync::{Arc, Weak};
use talpid_types::ErrorExt;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize route monitor")]
    StartMonitorError(#[error(source)] talpid_routing::PlatformError),
}

pub struct MonitorHandle {
    _notify_tx: Arc<UnboundedSender<bool>>,
}

impl MonitorHandle {
    /// Host is considered to be offline if the IPv4 internet is considered to be unreachable by the
    /// given reachability flags *or* there are no active physical interfaces.
    pub async fn host_is_offline(&self) -> bool {
        !exists_non_tunnel_default_route().await
    }
}

async fn exists_non_tunnel_default_route() -> bool {
    match talpid_routing::get_default_routes().await {
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
    let mut monitor = talpid_routing::listen_for_default_route_changes()?;

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
