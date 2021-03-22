#![cfg(windows)]

use super::SharedTunnelStateValues;
use crate::{
    split_tunnel::{self, SplitTunnel},
    tunnel::TunnelMetadata,
    winnet::{self, get_best_default_route, interface_luid_to_ip, WinNetAddrFamily},
};
use lazy_static::lazy_static;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::{Arc, Mutex},
};
use talpid_types::{BoxedError, ErrorExt};

lazy_static! {
    static ref RESERVED_IP_V4: Ipv4Addr = "192.0.2.123".parse().unwrap();
}


pub(super) fn update_split_tunnel_addresses(
    metadata: Option<&TunnelMetadata>,
    shared_values: &mut SharedTunnelStateValues,
) -> Result<(), BoxedError> {
    let mut tunnel_ipv4 = None;
    let mut tunnel_ipv6 = None;

    if let Some(metadata) = metadata {
        for ip in &metadata.ips {
            match ip {
                IpAddr::V4(address) => tunnel_ipv4 = Some(address.clone()),
                IpAddr::V6(address) => tunnel_ipv6 = Some(address.clone()),
            }
        }
    }

    // Identify IP address that gives us Internet access
    let internet_ipv4 = get_best_default_route(WinNetAddrFamily::IPV4)
        .map_err(BoxedError::new)?
        .map(|route| interface_luid_to_ip(WinNetAddrFamily::IPV4, route.interface_luid))
        .transpose()
        .map_err(BoxedError::new)?
        .flatten();
    let internet_ipv6 = get_best_default_route(WinNetAddrFamily::IPV6)
        .map_err(BoxedError::new)?
        .map(|route| interface_luid_to_ip(WinNetAddrFamily::IPV6, route.interface_luid))
        .transpose()
        .map_err(BoxedError::new)?
        .flatten();

    let tunnel_ipv4 = tunnel_ipv4.unwrap_or(*RESERVED_IP_V4);
    let internet_ipv4 = Ipv4Addr::from(internet_ipv4.unwrap_or_default());
    let internet_ipv6 = internet_ipv6.map(|addr| Ipv6Addr::from(addr));

    let context = SplitTunnelDefaultRouteChangeHandlerContext::new(
        shared_values.split_tunnel.clone(),
        tunnel_ipv4,
        tunnel_ipv6,
        internet_ipv4,
        internet_ipv6,
    );

    shared_values.st_route_handler = None;

    shared_values
        .split_tunnel
        .lock()
        .expect("Thread unexpectedly panicked while holding the mutex")
        .register_ips(tunnel_ipv4, tunnel_ipv6, internet_ipv4, internet_ipv6)
        .map_err(BoxedError::new)?;

    shared_values.st_route_handler = Some(
        shared_values
            .route_manager
            .add_default_route_callback(Some(split_tunnel_default_route_change_handler), context)
            .map_err(BoxedError::new)?,
    );

    Ok(())
}

pub(super) fn clear_split_tunnel_addresses(shared_values: &mut SharedTunnelStateValues) {
    shared_values.st_route_handler = None;

    if let Err(error) = shared_values
        .split_tunnel
        .lock()
        .expect("Thread unexpectedly panicked while holding the mutex")
        .register_ips(
            Ipv4Addr::new(0, 0, 0, 0),
            None,
            Ipv4Addr::new(0, 0, 0, 0),
            None,
        )
    {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to unregister IP addresses")
        );
    }
}

pub unsafe extern "system" fn split_tunnel_default_route_change_handler(
    event_type: winnet::WinNetDefaultRouteChangeEventType,
    address_family: WinNetAddrFamily,
    default_route: winnet::WinNetDefaultRoute,
    ctx: *mut libc::c_void,
) {
    // Update the "internet interface" IP when best default route changes
    let ctx = &mut *(ctx as *mut SplitTunnelDefaultRouteChangeHandlerContext);

    let result = match event_type {
        winnet::WinNetDefaultRouteChangeEventType::DefaultRouteChanged => {
            let ip = interface_luid_to_ip(address_family.clone(), default_route.interface_luid);

            // TODO: Should we block here?
            let ip = match ip {
                Ok(Some(ip)) => ip,
                Ok(None) => {
                    log::error!("Failed to obtain new default route address: none found",);
                    // Early return
                    return;
                }
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to obtain new default route address")
                    );
                    // Early return
                    return;
                }
            };

            match address_family {
                WinNetAddrFamily::IPV4 => {
                    let ip = Ipv4Addr::from(ip);
                    ctx.internet_ipv4 = ip;
                }
                WinNetAddrFamily::IPV6 => {
                    let ip = Ipv6Addr::from(ip);
                    ctx.internet_ipv6 = Some(ip);
                }
            }

            ctx.register_ips()
        }
        // no default route
        winnet::WinNetDefaultRouteChangeEventType::DefaultRouteRemoved => {
            match address_family {
                WinNetAddrFamily::IPV4 => {
                    ctx.internet_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
                }
                WinNetAddrFamily::IPV6 => {
                    ctx.internet_ipv6 = None;
                }
            }
            ctx.register_ips()
        }
    };

    if let Err(error) = result {
        // TODO: Should we block here?
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to register new addresses in split tunnel driver")
        );
    }
}

struct SplitTunnelDefaultRouteChangeHandlerContext {
    split_tunnel: Arc<Mutex<SplitTunnel>>,
    pub tunnel_ipv4: Ipv4Addr,
    pub tunnel_ipv6: Option<Ipv6Addr>,
    pub internet_ipv4: Ipv4Addr,
    pub internet_ipv6: Option<Ipv6Addr>,
}

impl SplitTunnelDefaultRouteChangeHandlerContext {
    pub fn new(
        split_tunnel: Arc<Mutex<SplitTunnel>>,
        tunnel_ipv4: Ipv4Addr,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Ipv4Addr,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> Self {
        SplitTunnelDefaultRouteChangeHandlerContext {
            split_tunnel,
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        }
    }

    pub fn register_ips(&self) -> Result<(), split_tunnel::Error> {
        let split_tunnel = self
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex");
        split_tunnel.register_ips(
            self.tunnel_ipv4,
            self.tunnel_ipv6,
            self.internet_ipv4,
            self.internet_ipv6,
        )
    }
}
