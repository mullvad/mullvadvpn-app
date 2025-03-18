//! Functions for handling default interfaces/routes

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

    let default_interface = match (&v4_default, &v6_default) {
        (Some(v4_default), Some(v6_default)) => {
            if v4_default.interface != v6_default.interface {
                return Err(Error::DefaultInterfaceMismatch);
            }
            v4_default.interface.clone()
        }
        (Some(default), None) | (None, Some(default)) => default.interface.clone(),
        (None, None) => return Err(Error::NoDefaultInterface),
    };

    let default_v4 = if let Some(v4_gateway) = v4_gateway {
        v4_default.map(|v4_default| DefaultInterfaceAddrs {
            source_ip: match v4_default.ip {
                IpAddr::V4(addr) => addr,
                _ => unreachable!("unexpected IP address type"),
            },
            gateway_address: v4_gateway.mac_address,
        })
    } else {
        log::debug!("Missing V4 gateway");
        None
    };
    let default_v6 = if let Some(v6_gateway) = v6_gateway {
        v6_default.map(|v6_default| DefaultInterfaceAddrs {
            source_ip: match v6_default.ip {
                IpAddr::V6(addr) => addr,
                _ => unreachable!("unexpected IP address type"),
            },
            gateway_address: v6_gateway.mac_address,
        })
    } else {
        log::debug!("Missing V6 gateway");
        None
    };

    Ok(DefaultInterface {
        name: default_interface,
        v4_addrs: default_v4,
        v6_addrs: default_v6,
    })
}
