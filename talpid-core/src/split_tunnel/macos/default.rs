//! Functions for handling default interfaces/routes

use futures::{channel::mpsc, StreamExt};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use talpid_routing::{MacAddress, RouteManagerHandle};
use talpid_types::ErrorExt;

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
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultInterface {
    /// Interface name
    pub name: String,
    /// MAC/Hardware address of the gateway
    pub v4_addrs: Option<DefaultInterfaceAddrs<Ipv4Addr>>,
    /// MAC/Hardware address of the gateway
    pub v6_addrs: Option<DefaultInterfaceAddrs<Ipv6Addr>>,
}

/// Interface name, addresses, and gateway
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultInterfaceAddrs<IpType> {
    /// Source IP address for excluded apps
    pub source_ip: IpType,
    /// MAC/Hardware address of the gateway
    pub gateway_address: MacAddress,
}

/// Task that returns the new default interface when it changes
pub async fn default_interface_change_listener(
    route_manager: RouteManagerHandle,
) -> mpsc::UnboundedReceiver<DefaultInterface> {
    let (tx, rx) = mpsc::unbounded();

    let mut prev_interface = None;

    let mut default_route_stream = match route_manager.default_route_listener().await {
        Ok(routes) => routes,
        Err(err) => {
            log::error!(
                "{}",
                err.display_chain_with_msg("Failed to start default route listener")
            );
            return rx;
        }
    };

    tokio::spawn(async move {
        while let Some(_route_event) = default_route_stream.next().await {
            match super::default::get_default_interface(&route_manager).await {
                Ok(default_interface) if prev_interface.as_ref() != Some(&default_interface) => {
                    let _ = tx.unbounded_send(default_interface.clone());
                    prev_interface = Some(default_interface);
                }
                Ok(_) => (),
                Err(err) => {
                    log::error!(
                        "{}",
                        err.display_chain_with_msg("Failed to retrieve default interface")
                    );
                }
            }
        }
    });

    rx
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
