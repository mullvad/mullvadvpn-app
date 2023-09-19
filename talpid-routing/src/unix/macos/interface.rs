use ipnetwork::IpNetwork;
use libc::{if_indextoname, IFNAMSIZ};
use nix::net::if_::{if_nametoindex, InterfaceFlags};
use std::{
    ffi::{CStr, CString},
    io,
    net::{Ipv4Addr, Ipv6Addr},
};
use system_configuration::{
    core_foundation::string::CFString,
    network_configuration::{SCNetworkService, SCNetworkSet},
    preferences::SCPreferences,
};

use super::{
    data::{Destination, RouteMessage},
    watch::RoutingTable,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Family {
    V4,
    V6,
}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Family::V4 => f.write_str("V4"),
            Family::V6 => f.write_str("V6"),
        }
    }
}

impl From<Family> for IpNetwork {
    fn from(fam: Family) -> Self {
        match fam {
            Family::V4 => IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap(),
            Family::V6 => IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap(),
        }
    }
}

/// Retrieve the current unscoped default route. That is the only default route that does not have
/// the IF_SCOPE flag set, if such a route exists.
///
/// # Note
///
/// For some reason, the socket sometimes returns a route with the IF_SCOPE flag set, if there also
/// exists a scoped route for the same interface. This does not occur if there is no unscoped route,
/// so we can still rely on it.
pub async fn get_unscoped_default_route(
    routing_table: &mut RoutingTable,
    family: Family,
) -> Option<RouteMessage> {
    let mut msg = RouteMessage::new_route(Destination::Network(IpNetwork::from(family)));
    msg = msg.set_gateway_route(true);

    let route = routing_table
        .get_route(&msg)
        .await
        .unwrap_or_else(|error| {
            log::error!("Failed to retrieve unscoped default route: {error}");
            None
        })?;

    let idx = u32::from(route.interface_index());
    if idx != 0 {
        let mut ifname = [0u8; IFNAMSIZ];

        // SAFETY: The buffer is large to contain any interface name.
        if !unsafe { if_indextoname(idx, ifname.as_mut_ptr() as _) }.is_null() {
            let ifname = CStr::from_bytes_until_nul(&ifname).unwrap();
            let name = ifname.to_str().expect("expected ascii");

            // Ignore the unscoped route if its interface is not "active"
            if !is_active_interface(name, family).unwrap_or(true) {
                return None;
            }
        }
    }

    Some(route)
}

/// Retrieve the best current default route. That is the first scoped default route, ordered by
/// network service order, and with interfaces filtered out if they do not have valid IP addresses
/// assigned.
///
/// # Note
///
/// The tunnel interface is not even listed in the service order, so it will be skipped.
pub async fn get_best_default_route(
    routing_table: &mut RoutingTable,
    family: Family,
) -> Option<RouteMessage> {
    let mut msg = RouteMessage::new_route(Destination::Network(IpNetwork::from(family)));
    msg = msg.set_gateway_route(true);

    for iface in network_service_order() {
        let iface_bytes = match CString::new(iface.as_bytes()) {
            Ok(name) => name,
            Err(error) => {
                log::error!("Invalid interface name: {iface}, {error}");
                continue;
            }
        };

        // Get interface ID
        let index = match if_nametoindex(iface_bytes.as_c_str()) {
            Ok(index) => index,
            Err(_error) => {
                continue;
            }
        };

        // Request ifscoped default route for this interface
        let route_msg = msg.clone().set_ifscope(u16::try_from(index).unwrap());
        if let Ok(Some(route)) = routing_table.get_route(&route_msg).await {
            if is_active_interface(&iface, family).unwrap_or(true) {
                return Some(route);
            }
        }
    }

    None
}

fn network_service_order() -> Vec<String> {
    let prefs = SCPreferences::default(&CFString::new("talpid-routing"));
    let services = SCNetworkService::get_services(&prefs);
    let set = SCNetworkSet::new(&prefs);
    let service_order = set.service_order();

    service_order
        .iter()
        .filter_map(|service_id| {
            services
                .iter()
                .find(|service| service.id().as_ref() == Some(&*service_id))
                .and_then(|service| service.network_interface()?.bsd_name())
                .map(|cf_name| cf_name.to_string())
        })
        .collect::<Vec<_>>()
}

/// Return whether the given interface has an assigned (unicast) IP address.
fn is_active_interface(interface_name: &str, family: Family) -> io::Result<bool> {
    let required_link_flags: InterfaceFlags = InterfaceFlags::IFF_UP | InterfaceFlags::IFF_RUNNING;
    let has_ip_addr = nix::ifaddrs::getifaddrs()?
        .filter(|addr| (addr.flags & required_link_flags) == required_link_flags)
        .filter(|addr| addr.interface_name == interface_name)
        .any(|addr| {
            if let Some(addr) = addr.address {
                // Check if family matches; ignore if link-local address
                match family {
                    Family::V4 => matches!(addr.as_sockaddr_in(), Some(addr_in) if is_routable_v4(&Ipv4Addr::from(addr_in.ip()))),
                    Family::V6 => {
                        matches!(addr.as_sockaddr_in6(), Some(addr_in) if is_routable_v6(&addr_in.ip()))
                    }
                }
            } else {
                false
            }
        });
    Ok(has_ip_addr)
}

fn is_routable_v4(addr: &Ipv4Addr) -> bool {
    !addr.is_unspecified() && !addr.is_loopback() && !addr.is_link_local()
}

fn is_routable_v6(addr: &Ipv6Addr) -> bool {
    !addr.is_unspecified()
    && !addr.is_loopback()
    // !(link local)
    && (addr.segments()[0] & 0xffc0) != 0xfe80
}
