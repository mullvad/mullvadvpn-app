//! Functions for handling default interfaces/routes

use nix::{net::if_::InterfaceFlags, sys::socket::AddressFamily};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use talpid_routing::{MacAddress, RouteManagerHandle};

/// Interface errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to get default routes
    #[error("Failed to get default routes")]
    GetDefaultRoutes(#[source] talpid_routing::Error),
    /// Failed to get default gateways
    #[error("Failed to get default gateways")]
    GetDefaultGateways(#[source] talpid_routing::Error),
    /// Failed to get interface addresses
    #[error("Failed to get interface addresses")]
    GetInterfaceAddresses(#[source] nix::Error),
    /// Found no suitable default interface
    #[error("Found no suitable default interface")]
    NoDefaultInterface,
    /// Using different interfaces for IPv4 and IPv6 is not supported
    #[error("Using different interfaces for IPv4 and IPv6 is not supported")]
    DefaultInterfaceMismatch,
}

/// Interface name, addresses, and gateway
#[derive(Debug, Clone)]
pub struct DefaultInterface {
    /// Interface name
    pub name: String,
    /// MAC/Hardware address of the gateway
    pub v4_addrs: Option<DefaultInterfaceAddrs<Ipv4Addr>>,
    /// MAC/Hardware address of the gateway
    pub v6_addrs: Option<DefaultInterfaceAddrs<Ipv6Addr>>,
}

/// Interface name, addresses, and gateway
#[derive(Debug, Clone)]
pub struct DefaultInterfaceAddrs<IpType> {
    /// Source IP address for excluded apps
    pub source_ip: IpType,
    /// MAC/Hardware address of the gateway
    pub gateway_address: MacAddress,
}

pub async fn get_default_interface(
    route_manager: &RouteManagerHandle,
) -> Result<DefaultInterface, Error> {
    let (v4_default, v6_default) = route_manager
        .get_default_routes()
        .await
        .map_err(Error::GetDefaultRoutes)?;
    let (v4_gateway, v6_gateway) = route_manager
        .get_default_gateway()
        .await
        .map_err(Error::GetDefaultGateways)?;

    let default_interface = match (v4_default, v6_default) {
        (Some(v4_default), Some(v6_default)) => {
            let v4_name = v4_default
                .get_node()
                .get_device()
                .expect("missing device on default route");
            let v6_name = v6_default
                .get_node()
                .get_device()
                .expect("missing device on default route");
            if v4_name != v6_name {
                return Err(Error::DefaultInterfaceMismatch);
            }
            v4_name.to_owned()
        }
        (Some(default), None) | (None, Some(default)) => default
            .get_node()
            .get_device()
            .expect("missing device on default route")
            .to_owned(),
        (None, None) => return Err(Error::NoDefaultInterface),
    };

    let default_v4 = if let Some(v4_gateway) = v4_gateway {
        match get_interface_ip(&default_interface, AddressFamily::Inet) {
            Ok(Some(ip)) => Some(DefaultInterfaceAddrs {
                source_ip: match ip {
                    IpAddr::V4(addr) => addr,
                    _ => unreachable!("unexpected address type"),
                },
                gateway_address: v4_gateway.mac_address,
            }),
            Ok(None) => None,
            Err(error) => {
                log::error!("Failed to obtain interface IP for {default_interface}: {error}");
                None
            }
        }
    } else {
        None
    };
    let default_v6 = if let Some(v6_gateway) = v6_gateway {
        match get_interface_ip(&default_interface, AddressFamily::Inet6) {
            Ok(Some(ip)) => Some(DefaultInterfaceAddrs {
                source_ip: match ip {
                    IpAddr::V6(addr) => addr,
                    _ => unreachable!("unexpected address type"),
                },
                gateway_address: v6_gateway.mac_address,
            }),
            Ok(None) => None,
            Err(error) => {
                log::error!("Failed to obtain interface IP for {default_interface}: {error}");
                None
            }
        }
    } else {
        None
    };

    Ok(DefaultInterface {
        name: default_interface,
        v4_addrs: default_v4,
        v6_addrs: default_v6,
    })
}

fn get_interface_ip(interface_name: &str, family: AddressFamily) -> Result<Option<IpAddr>, Error> {
    let required_link_flags: InterfaceFlags = InterfaceFlags::IFF_UP | InterfaceFlags::IFF_RUNNING;
    let ip_addr = nix::ifaddrs::getifaddrs()
        .map_err(Error::GetInterfaceAddresses)?
        .filter(|addr| (addr.flags & required_link_flags) == required_link_flags)
        .filter(|addr| addr.interface_name == interface_name)
        .find_map(|addr| {
            let Some(addr) = addr.address else {
                return None;
            };
            // Check if family matches; ignore if link-local address
            match family {
                AddressFamily::Inet => match addr.as_sockaddr_in() {
                    Some(addr_in) => {
                        let addr_in = Ipv4Addr::from(addr_in.ip());
                        if is_routable_v4(&addr_in) {
                            Some(IpAddr::from(addr_in))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                AddressFamily::Inet6 => match addr.as_sockaddr_in6() {
                    Some(addr_in) => {
                        let addr_in = Ipv6Addr::from(addr_in.ip());
                        if is_routable_v6(&addr_in) {
                            Some(IpAddr::from(addr_in))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            }
        });
    Ok(ip_addr)
}

fn is_routable_v4(addr: &Ipv4Addr) -> bool {
    !addr.is_unspecified() && !addr.is_loopback() && !addr.is_link_local()
}

fn is_routable_v6(addr: &Ipv6Addr) -> bool {
    !addr.is_unspecified() && !addr.is_loopback() && !is_link_local_v6(addr)
}

fn is_link_local_v6(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}
