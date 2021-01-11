use crate::tunnel_state_machine::TunnelCommand;
use futures::{
    channel::{mpsc::UnboundedSender, oneshot},
    FutureExt, StreamExt, TryStreamExt,
};
use netlink_packet_route::rtnl::route::nlas::Nla as RouteNla;
use rtnetlink::{
    constants::{RTMGRP_IPV4_IFADDR, RTMGRP_IPV6_IFADDR, RTMGRP_LINK, RTMGRP_NOTIFY},
    sys::SocketAddr,
    Handle, IpVersion,
};
use std::{io, net::Ipv4Addr, sync::Weak};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to get list of IP links")]
    GetLinksError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to get list of IP addresses")]
    GetAddressesError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to get a route for an arbitrary IP address")]
    GetRouteError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to connect to netlink socket")]
    NetlinkConnectionError(#[error(source)] io::Error),

    #[error(display = "Failed to connect to bind to netlink socket")]
    BindError(#[error(source)] io::Error),

    #[error(display = "Failed to start listening on netlink socket")]
    NetlinkBindError(#[error(source)] io::Error),

    #[error(display = "Error while processing netlink messages")]
    MonitorNetlinkError,

    #[error(display = "Netlink connection has unexpectedly disconnected")]
    NetlinkDisconnected,

    #[error(display = "Failed to initialize event loop")]
    EventLoopError(#[error(source)] io::Error),
}

pub struct MonitorHandle {
    handle: rtnetlink::Handle,
    _stop_connection_tx: oneshot::Sender<()>,
}

// Mullvad API's public IP address, correct at the time of writing, but any public IP address will
// work.
const PUBLIC_INTERNET_ADDRESS: Ipv4Addr = Ipv4Addr::new(193, 138, 218, 78);

impl MonitorHandle {
    pub async fn is_offline(&mut self) -> bool {
        match public_ip_unreachable(&self.handle).await {
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

pub async fn spawn_monitor(sender: Weak<UnboundedSender<TunnelCommand>>) -> Result<MonitorHandle> {
    let (mut connection, handle, mut messages) =
        rtnetlink::new_connection().map_err(Error::NetlinkConnectionError)?;

    let mgroup_flags = RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR | RTMGRP_LINK | RTMGRP_NOTIFY;
    let addr = SocketAddr::new(0, mgroup_flags);

    connection
        .socket_mut()
        .bind(&addr)
        .map_err(Error::BindError)?;

    let (stop_connection_tx, stop_rx) = oneshot::channel();

    // Connection will be closed once the channel is dropped
    tokio::spawn(async {
        futures::select! {
            _ = connection.fuse() => (),
            _ = stop_rx.fuse() => (),
        }
    });
    let mut is_offline = public_ip_unreachable(&handle).await?;

    let monitor_handle = MonitorHandle {
        handle: handle.clone(),
        _stop_connection_tx: stop_connection_tx,
    };


    tokio::spawn(async move {
        while let Some(_new_message) = messages.next().await {
            match sender.upgrade() {
                Some(sender) => {
                    let new_offline_state = public_ip_unreachable(&handle).await.unwrap_or(false);
                    if new_offline_state != is_offline {
                        is_offline = new_offline_state;
                        let _ = sender.unbounded_send(TunnelCommand::IsOffline(is_offline));
                    }
                }
                None => return,
            }
        }
    });


    Ok(monitor_handle)
}


async fn public_ip_unreachable(handle: &Handle) -> Result<bool> {
    let mut request = handle.route().get(IpVersion::V4);
    let message = request.message_mut();
    message
        .nlas
        .push(RouteNla::Mark(crate::linux::TUNNEL_FW_MARK));
    message.nlas.push(RouteNla::Destination(
        PUBLIC_INTERNET_ADDRESS.octets().to_vec(),
    ));
    message.header.destination_prefix_length = 32;
    let mut stream = request.execute();
    while let Some(message) = stream
        .try_next()
        .await
        .map_err(failure::Fail::compat)
        .map_err(Error::GetRouteError)?
    {
        for nla in message.nlas.iter() {
            if let RouteNla::Gateway(_) | RouteNla::Oif(_) = nla {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod test {
    use super::*;
    use rtnetlink::{
        constants::{RTMGRP_IPV4_IFADDR, RTMGRP_IPV6_IFADDR, RTMGRP_LINK, RTMGRP_NOTIFY},
        sys::SocketAddr,
    };

    #[test]
    fn test_route_table_query() {
        let mut runtime = tokio::runtime::Runtime::new().expect("failed to initialize runtime");
        let (mut connection, handle, _) = runtime.block_on(async {
            rtnetlink::new_connection()
                .map_err(Error::NetlinkConnectionError)
                .expect("Failed to create a netlink connection")
        });

        let mgroup_flags = RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR | RTMGRP_LINK | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);

        connection.socket_mut().bind(&addr).unwrap();
        runtime.spawn(connection);

        runtime
            .block_on(public_ip_unreachable(&handle))
            .expect("Failed to query routing table");
    }
}
