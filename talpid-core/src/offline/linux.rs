use futures::{channel::mpsc::UnboundedSender, StreamExt};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};
use talpid_routing::{self, RouteManagerHandle};
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "The route manager returned an error")]
    RouteManagerError(#[error(source)] talpid_routing::Error),
}

pub struct MonitorHandle {
    route_manager: RouteManagerHandle,
    fwmark: Option<u32>,
    _notify_tx: Arc<UnboundedSender<bool>>,
}

const PUBLIC_INTERNET_ADDRESS_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(193, 138, 218, 78));
const PUBLIC_INTERNET_ADDRESS_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6));

impl MonitorHandle {
    pub async fn host_is_offline(&self) -> bool {
        match public_ip_unreachable(&self.route_manager, self.fwmark).await {
            Ok(is_offline) => is_offline,
            Err(err) => {
                log::error!(
                    "Failed to verify offline state: {}. Presuming connectivity",
                    err
                );
                false
            }
        }
    }
}

pub async fn spawn_monitor(
    notify_tx: UnboundedSender<bool>,
    route_manager: RouteManagerHandle,
    fwmark: Option<u32>,
) -> Result<MonitorHandle> {
    let mut is_offline = public_ip_unreachable(&route_manager, fwmark).await?;

    let mut listener = route_manager
        .change_listener()
        .await
        .map_err(Error::RouteManagerError)?;

    let notify_tx = Arc::new(notify_tx);
    let sender = Arc::downgrade(&notify_tx);
    let monitor_handle = MonitorHandle {
        route_manager: route_manager.clone(),
        fwmark,
        _notify_tx: notify_tx,
    };

    tokio::spawn(async move {
        while let Some(_event) = listener.next().await {
            match sender.upgrade() {
                Some(sender) => {
                    let new_offline_state = public_ip_unreachable(&route_manager, fwmark)
                        .await
                        .unwrap_or_else(|err| {
                            log::error!(
                                "{}",
                                err.display_chain_with_msg("Failed to infer offline state")
                            );
                            false
                        });
                    if new_offline_state != is_offline {
                        is_offline = new_offline_state;
                        let _ = sender.unbounded_send(is_offline);
                    }
                }
                None => return,
            }
        }
    });

    Ok(monitor_handle)
}

async fn public_ip_unreachable(handle: &RouteManagerHandle, fwmark: Option<u32>) -> Result<bool> {
    Ok(handle
        .get_destination_route(PUBLIC_INTERNET_ADDRESS_V4, fwmark)
        .await
        .map_err(Error::RouteManagerError)?
        .is_none()
        && handle
            .get_destination_route(PUBLIC_INTERNET_ADDRESS_V6, fwmark)
            .await
            .unwrap_or(None)
            .is_none())
}
