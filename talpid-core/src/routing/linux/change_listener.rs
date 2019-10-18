use super::{
    super::{Node, Route},
    RouteChange,
};
use futures::{future::Either, sync::mpsc, Async, Future, Stream};
use std::{collections::BTreeMap, io, net::IpAddr};

use netlink_packet::{
    LinkMessage, LinkNla, NetlinkMessage, NetlinkPayload, RouteMessage, RouteNla, RtnlMessage,
};
use netlink_sys::SocketAddr;
use rtnetlink::constants::{
    AF_INET, AF_INET6, RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE, RTMGRP_LINK, RTMGRP_NOTIFY,
};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Netlink connection failed")]
    NetlinkError(#[error(source)] failure::Compat<rtnetlink::Error>),
    #[error(display = "Netlink protocol error")]
    NetlinkProtocolError(#[error(source)] failure::Compat<netlink_proto::Error>),
    #[error(display = "Failed to open a netlink connection")]
    ConnectError(#[error(source)] io::Error),
    #[error(display = "Route without a valid node")]
    InvalidRoute,
    #[error(display = "Invalid length of byte buffer for IP address")]
    InvalidIpBytes,
    #[error(display = "Invalid network prefix")]
    InvalidNetworkPrefix(#[error(source)] ipnetwork::IpNetworkError),
    #[error(display = "Unknown device index - {}", _0)]
    UnknownDeviceIndex(u32),
    #[error(display = "Failed to bind netlink socket")]
    BindError(#[error(source)] io::Error),
    #[error(display = "Netlink connection stopped sending messages")]
    NetlinkConnectionClosed,
}

type Result<T> = std::result::Result<T, Error>;

pub(super) struct RouteChangeListener {
    connection: rtnetlink::Connection,
    messages: mpsc::UnboundedReceiver<NetlinkMessage>,
    iface_map: BTreeMap<u32, String>,
}

impl RouteChangeListener {
    pub fn new() -> Result<Self> {
        let (mut connection, handle, messages) =
            rtnetlink::new_connection_with_messages().map_err(Error::ConnectError)?;

        let mgroup_flags = RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE | RTMGRP_LINK | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);
        connection
            .socket_mut()
            .bind(&addr)
            .map_err(Error::BindError)?;

        let (iface_map, connection) = Self::initialize_link_map(connection, handle)?;

        Ok(Self {
            connection,
            messages,
            iface_map,
        })
    }

    fn map_netlink_to_route_change(&mut self, msg: NetlinkMessage) -> Result<Option<RouteChange>> {
        match msg.payload {
            NetlinkPayload::Rtnl(RtnlMessage::NewLink(new_link)) => {
                if let Some((idx, name)) = Self::map_iface_name_to_idx(new_link) {
                    self.iface_map.insert(idx, name);
                }
                Ok(None)
            }
            NetlinkPayload::Rtnl(RtnlMessage::DelLink(old_link)) => {
                if let Some((idx, _)) = Self::map_iface_name_to_idx(old_link) {
                    self.iface_map.remove(&idx);
                }
                Ok(None)
            }

            NetlinkPayload::Rtnl(RtnlMessage::NewRoute(new_route)) => {
                self.get_route(new_route).map(RouteChange::Add).map(Some)
            }
            NetlinkPayload::Rtnl(RtnlMessage::DelRoute(old_route)) => {
                self.get_route(old_route).map(RouteChange::Remove).map(Some)
            }
            _ => Ok(None),
        }
    }

    // Tries to coax a Route out of a RouteMessage
    fn get_route(&self, msg: RouteMessage) -> Result<Route> {
        let mut prefix = None;
        let mut node_addr = None;
        let mut device = None;
        let mut metric = None;
        let mut gateway = None;

        let destination_length = msg.header.destination_length;
        let af_spec = msg.header.address_family;

        for nla in msg.nlas.iter() {
            match nla {
                RouteNla::Oif(device_idx) => {
                    match self.iface_map.get(&device_idx) {
                        Some(device_name) => device = Some(device_name.to_string()),
                        None => {
                            return Err(Error::UnknownDeviceIndex(*device_idx));
                        }
                    };
                }

                RouteNla::Via(addr) => {
                    node_addr = Self::parse_ip(&addr).map(Some)?;
                }

                RouteNla::Destination(addr) => {
                    prefix = Self::parse_ip(&addr)
                        .and_then(|ip| {
                            ipnetwork::IpNetwork::new(ip, destination_length)
                                .map_err(Error::InvalidNetworkPrefix)
                        })
                        .map(Some)?;
                }

                // gateway NLAs indicate that this is actually a default route
                RouteNla::Gateway(gateway_ip) => {
                    gateway = Self::parse_ip(&gateway_ip).map(Some)?;
                }

                RouteNla::Priority(priority) => {
                    metric = Some(*priority);
                }
                _ => continue,
            }
        }

        // when a gateway is specified but prefix is none, then this is a default route
        if prefix.is_none() && gateway.is_some() {
            prefix = match af_spec as u16 {
                AF_INET => Some("0.0.0.0/0".parse().expect("failed to parse ipnetwork")),
                AF_INET6 => Some("::/0".parse().expect("failed to parse ipnetwork")),
                _ => None,
            };
        }

        if device.is_none() && node_addr.is_none() || prefix.is_none() {
            return Err(Error::InvalidRoute);
        }


        let node = Node {
            ip: node_addr,
            device,
        };

        Ok(Route {
            node,
            prefix: prefix.unwrap(),
            metric,
        })
    }

    fn map_iface_name_to_idx(msg: LinkMessage) -> Option<(u32, String)> {
        let index = msg.header.index;
        for nla in msg.nlas {
            match nla {
                LinkNla::IfName(name) => return Some((index, name)),
                _ => continue,
            }
        }
        None
    }

    fn parse_ip(bytes: &[u8]) -> Result<IpAddr> {
        if bytes.len() == 4 {
            let mut ipv4_bytes = [0u8; 4];
            ipv4_bytes.copy_from_slice(bytes);
            Ok(IpAddr::from(ipv4_bytes))
        } else if bytes.len() == 16 {
            let mut ipv6_bytes = [0u8; 16];
            ipv6_bytes.copy_from_slice(bytes);
            Ok(IpAddr::from(ipv6_bytes))
        } else {
            log::error!("Expected either 4 or 16 bytes, got {} bytes", bytes.len());
            Err(Error::InvalidIpBytes)
        }
    }

    pub fn initialize_link_map(
        connection: rtnetlink::Connection,
        handle: rtnetlink::Handle,
    ) -> Result<(BTreeMap<u32, String>, rtnetlink::Connection)> {
        let request = handle
            .link()
            .get()
            .execute()
            .filter_map(Self::map_iface_name_to_idx)
            .collect();

        match connection.select2(request).wait() {
            Ok(Either::A(_)) => Err(Error::NetlinkConnectionClosed),
            Err(Either::A((error, _))) => {
                Err(Error::NetlinkProtocolError(failure::Fail::compat(error)))
            }
            Ok(Either::B((links, connection))) => Ok((links.into_iter().collect(), connection)),
            Err(Either::B((error, _))) => Err(Error::NetlinkError(failure::Fail::compat(error))),
        }
    }
}

impl Stream for RouteChangeListener {
    type Item = RouteChange;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<RouteChange>>> {
        self.connection
            .poll()
            .map_err(failure::Fail::compat)
            .map_err(Error::NetlinkProtocolError)?;

        loop {
            match futures::try_ready!(self
                .messages
                .poll()
                .map_err(|_| Error::NetlinkConnectionClosed))
            {
                Some(message) => {
                    if let Some(route_change) = self.map_netlink_to_route_change(message)? {
                        return Ok(Async::Ready(Some(route_change)));
                    };
                    continue;
                }
                None => {
                    return Err(Error::NetlinkConnectionClosed);
                }
            }
        }
    }
}
