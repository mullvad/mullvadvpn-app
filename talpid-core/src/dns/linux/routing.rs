use futures::{
    channel::mpsc::UnboundedSender, future::abortable, FutureExt, StreamExt, TryStream,
    TryStreamExt,
};
use netlink_packet_core::{NetlinkPayload, NLM_F_REQUEST};
use netlink_packet_route::{
    rtnl::route::nlas::Nla as RouteNla, NetlinkMessage, RouteFlags, RouteMessage, RtnlMessage,
};
use rtnetlink::{
    constants::{RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE, RTMGRP_NOTIFY},
    sys::SocketAddr,
    Handle, IpVersion,
};
use std::{
    collections::BTreeMap,
    fmt, io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

const PUBLIC_INTERNET_ADDRESS_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6));
const PUBLIC_INTERNET_ADDRESS_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to get a route for an arbitrary IP address")]
    GetRouteError(#[error(source)] failure::Compat<rtnetlink::Error>),

    #[error(display = "Failed to connect to bind to netlink socket")]
    BindError(#[error(source)] io::Error),

    #[error(display = "No netlink response for route query")]
    NoRouteError,

    #[error(display = "Route is missing an output interface")]
    RouteNoInterfaceError,
}

pub struct DnsRouteMonitor {
    _handle: rtnetlink::Handle,
    stop_tx: Option<futures::channel::oneshot::Sender<()>>,
}

impl Drop for DnsRouteMonitor {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DnsConfig {
    pub interface: u32,
    pub resolvers: Vec<IpAddr>,
}

impl fmt::Display for DnsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "interface index {}, resolvers:", self.interface)?;
        for server in &self.resolvers {
            write!(f, " {}", server)?;
        }
        Ok(())
    }
}

pub async fn spawn_monitor(
    destinations: Vec<IpAddr>,
    update_tx: UnboundedSender<BTreeMap<u32, DnsConfig>>,
) -> Result<(DnsRouteMonitor, BTreeMap<u32, DnsConfig>)> {
    let (mut connection, handle, messages) =
        rtnetlink::new_connection().expect("Failed to create a netlink connection");

    let mgroup_flags = RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE | RTMGRP_NOTIFY;
    let addr = SocketAddr::new(0, mgroup_flags);

    connection
        .socket_mut()
        .bind(&addr)
        .map_err(Error::BindError)?;

    let (abortable_connection, abort_connection) = abortable(connection);
    tokio::spawn(abortable_connection);

    let (stop_tx, stop_rx) = futures::channel::oneshot::channel();

    let monitor = DnsRouteMonitor {
        _handle: handle.clone(),
        stop_tx: Some(stop_tx),
    };

    let mut last_config = setup_configurations(&handle, &destinations).await?;
    let initial_config = last_config.clone();

    tokio::spawn(async move {
        let mut messages = messages.fuse();
        let mut stop_rx = stop_rx.fuse();
        loop {
            futures::select! {
                _new_message = messages.next() => {
                    match setup_configurations(&handle, &destinations).await {
                        Ok(new_config) => {
                            if last_config != new_config {
                                last_config = new_config.clone();
                                if update_tx.unbounded_send(new_config).is_err() {
                                    log::trace!("Stopping DNS monitor: channel is closed");
                                    break;
                                }
                            }
                        }
                        Err(error) => {
                            log::error!(
                                "{}",
                                error.display_chain_with_msg(
                                    "Failed to determine new DNS interface settings"
                                )
                            );
                        }
                    }
                },
                _ = stop_rx => break,
            }
        }
        abort_connection.abort();
    });

    Ok((monitor, initial_config))
}

async fn setup_configurations(
    handle: &Handle,
    destinations: &[IpAddr],
) -> Result<BTreeMap<u32, DnsConfig>> {
    let mut interface_to_destinations = BTreeMap::<u32, DnsConfig>::new();
    for destination in destinations {
        let interface = if destination.is_loopback() {
            get_default_route_interface(handle, get_ip_version(destination), true).await?
        } else {
            if crate::firewall::is_local_address(&destination) {
                get_destination_interface(handle, destination, true).await?
            } else {
                get_default_route_interface(handle, get_ip_version(destination), false).await?
            }
        };
        match interface {
            Some(iface) => {
                if let Some(config) = interface_to_destinations.get_mut(&iface) {
                    config.resolvers.push(*destination);
                } else {
                    interface_to_destinations.insert(
                        iface,
                        DnsConfig {
                            interface: iface,
                            resolvers: vec![*destination],
                        },
                    );
                }
            }
            None => {
                log::trace!(
                    "Ignoring DNS server that did not match to any interface: {}",
                    destination
                );
            }
        }
    }

    Ok(interface_to_destinations)
}

async fn get_default_route_interface(
    handle: &Handle,
    ip_version: IpVersion,
    set_mark: bool,
) -> Result<Option<u32>> {
    match ip_version {
        IpVersion::V4 => {
            get_destination_interface(handle, &PUBLIC_INTERNET_ADDRESS_V4, set_mark).await
        }
        IpVersion::V6 => {
            get_destination_interface(handle, &PUBLIC_INTERNET_ADDRESS_V6, set_mark).await
        }
    }
}

async fn get_destination_interface(
    handle: &Handle,
    destination: &IpAddr,
    set_mark: bool,
) -> Result<Option<u32>> {
    let mut request = handle.route().get(get_ip_version(destination));
    let octets = match destination {
        IpAddr::V4(address) => address.octets().to_vec(),
        IpAddr::V6(address) => address.octets().to_vec(),
    };
    let message = request.message_mut();
    if set_mark {
        message
            .nlas
            .push(RouteNla::Mark(crate::linux::TUNNEL_FW_MARK));
    }
    message.header.destination_prefix_length = 8u8 * (octets.len() as u8);
    message.header.flags = RouteFlags::RTM_F_FIB_MATCH;
    message.nlas.push(RouteNla::Destination(octets));
    let mut stream = execute_route_get_request(handle.clone(), message.clone());
    match stream.try_next().await {
        Ok(Some(route_msg)) => {
            for nla in &route_msg.nlas {
                if let RouteNla::Oif(interface) = nla {
                    return Ok(Some(*interface));
                }
            }
            Err(Error::RouteNoInterfaceError)
        }
        Ok(None) => Err(Error::NoRouteError),
        Err(rtnetlink::Error::NetlinkError(nl_err)) if nl_err.code == -libc::ENETUNREACH => {
            Ok(None)
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

fn get_ip_version(addr: &IpAddr) -> IpVersion {
    if addr.is_ipv4() {
        IpVersion::V4
    } else {
        IpVersion::V6
    }
}
