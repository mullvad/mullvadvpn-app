use futures::{channel::mpsc::UnboundedSender, StreamExt};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};
use talpid_routing::RouteManagerHandle;
use talpid_types::{net::Connectivity, ErrorExt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The route manager returned an error")]
    RouteManagerError(#[source] talpid_routing::Error),
}

pub struct MonitorHandle {
    route_manager: RouteManagerHandle,
    fwmark: Option<u32>,
    _notify_tx: Arc<UnboundedSender<Connectivity>>,
}

/// A non-local IPv4 address.
const PUBLIC_INTERNET_ADDRESS_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(193, 138, 218, 78));
/// A non-local IPv6 address.
const PUBLIC_INTERNET_ADDRESS_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6));

impl MonitorHandle {
    pub async fn connectivity(&self) -> Connectivity {
        check_connectivity(&self.route_manager, self.fwmark).await
    }
}

pub async fn spawn_monitor(
    notify_tx: UnboundedSender<Connectivity>,
    route_manager: RouteManagerHandle,
    fwmark: Option<u32>,
) -> Result<MonitorHandle> {
    let mut connectivity = check_connectivity(&route_manager, fwmark).await;

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
                    let new_connectivity = check_connectivity(&route_manager, fwmark).await;
                    if new_connectivity != connectivity {
                        connectivity = new_connectivity;
                        let _ = sender.unbounded_send(connectivity);
                    }
                }
                None => return,
            }
        }
    });

    Ok(monitor_handle)
}

async fn check_connectivity(handle: &RouteManagerHandle, fwmark: Option<u32>) -> Connectivity {
    let route_exists = |destination| async move {
        handle
            .get_destination_route(destination, fwmark)
            .await
            .map(|route| route.is_some())
    };

    match (
        route_exists(PUBLIC_INTERNET_ADDRESS_V4).await,
        route_exists(PUBLIC_INTERNET_ADDRESS_V6).await,
    ) {
        (Ok(true), Err(err)) => {
            // Errors for IPv6 likely mean it's disabled, so assume it's unavailable
            log::trace!(
                "{}",
                err.display_chain_with_msg(
                    "Failed to infer offline state for IPv6. Assuming it's unavailable"
                )
            );
            Connectivity::Online(talpid_types::net::IpAvailability::IpV4)
        }
        (Ok(ipv4), Ok(ipv6)) => Connectivity::new(ipv4, ipv6),
        // If we fail to retrieve the IPv4 route, always assume we're connected
        (Err(err), _) => {
            log::error!(
                "Failed to verify offline state: {}. Presuming connectivity",
                err
            );
            Connectivity::PresumeOnline
        }
    }
}
