use crate::tunnel_state_machine::TunnelCommand;
use futures::{
    channel::{mpsc::UnboundedSender, oneshot},
    FutureExt, StreamExt, TryStream, TryStreamExt,
};
use netlink_packet_core::{NetlinkPayload, NLM_F_REQUEST};
use netlink_packet_route::{
    rtnl::route::nlas::Nla as RouteNla, NetlinkMessage, RouteFlags, RouteMessage, RtnlMessage,
};
use rtnetlink::{
    constants::{
        RTMGRP_IPV4_IFADDR, RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_IFADDR, RTMGRP_IPV6_ROUTE, RTMGRP_LINK,
        RTMGRP_NOTIFY,
    },
    sys::SocketAddr,
    Handle, IpVersion,
};
use std::{io, net::Ipv4Addr, sync::Weak};
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to resolve output interface index")]
    GetLinkError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "No netlink response for output interface query")]
    NoLinkError,

    #[error(display = "Failed to get list of IP addresses")]
    GetAddressesError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to get a route for an arbitrary IP address")]
    GetRouteError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "No netlink response for route query")]
    NoRouteError,

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

    let mgroup_flags = RTMGRP_IPV4_IFADDR
        | RTMGRP_IPV4_ROUTE
        | RTMGRP_IPV6_IFADDR
        | RTMGRP_IPV6_ROUTE
        | RTMGRP_LINK
        | RTMGRP_NOTIFY;
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
                    let new_offline_state =
                        public_ip_unreachable(&handle).await.unwrap_or_else(|err| {
                            log::error!(
                                "{}",
                                err.display_chain_with_msg("Failed to infer offline state")
                            );
                            false
                        });
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
    message.header.flags = RouteFlags::RTM_F_LOOKUP_TABLE;
    let mut stream = execute_route_get_request(handle.clone(), message.clone());
    match stream.try_next().await {
        // Presance of any route implies connectivity, even if it's a loopback route
        Ok(Some(_)) => Ok(false),
        Ok(None) => Err(Error::NoRouteError),
        // ENETUNREACH implies that there exists no route that'd reach our random API address,
        // as such, the host is assumed to be offline
        Err(rtnetlink::Error::NetlinkError(nl_err)) if nl_err.code == -libc::ENETUNREACH => {
            Ok(true)
        }
        Err(err) => Err(Error::GetRouteError(failure::Fail::compat(err))),
    }
}

pub fn execute_route_get_request(
    mut handle: Handle,
    message: RouteMessage,
) -> impl TryStream<Ok = RouteMessage, Error = rtnetlink::Error> {
    use futures::future::{self, Either};
    use rtnetlink::Error;

    let mut req = NetlinkMessage::from(RtnlMessage::GetRoute(message));
    req.header.flags = NLM_F_REQUEST;

    match handle.request(req) {
        Ok(response) => Either::Left(response.map(move |msg| {
            let (header, payload) = msg.into_parts();
            match payload {
                NetlinkPayload::InnerMessage(RtnlMessage::NewRoute(msg)) => Ok(msg),
                NetlinkPayload::Error(err) => Err(Error::NetlinkError(err)),
                _ => Err(Error::UnexpectedMessage(NetlinkMessage::new(
                    header, payload,
                ))),
            }
        })),
        Err(e) => Either::Right(future::err::<RouteMessage, Error>(e).into_stream()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rtnetlink::{
        constants::{
            RTMGRP_IPV4_IFADDR, RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_IFADDR, RTMGRP_IPV6_ROUTE,
            RTMGRP_LINK, RTMGRP_NOTIFY,
        },
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

        let mgroup_flags = RTMGRP_IPV4_IFADDR
            | RTMGRP_IPV4_ROUTE
            | RTMGRP_IPV6_IFADDR
            | RTMGRP_IPV6_ROUTE
            | RTMGRP_LINK
            | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);

        connection.socket_mut().bind(&addr).unwrap();
        runtime.spawn(connection);

        runtime
            .block_on(public_ip_unreachable(&handle))
            .expect("Failed to query routing table");
    }
}
