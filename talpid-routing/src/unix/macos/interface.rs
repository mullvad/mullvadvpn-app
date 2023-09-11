use nix::net::if_::{if_nametoindex, InterfaceFlags};
use std::{
    ffi::CString,
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

/// Attempt to retrieve the best current default route.
/// Note: The tunnel interface is not even listed in the service order, so it will be skipped.
pub async fn get_best_default_route(
    routing_table: &mut RoutingTable,
    family: Family,
) -> Option<RouteMessage> {
    let destination = match family {
        Family::V4 => super::v4_default(),
        Family::V6 => super::v6_default(),
    };

    let mut msg = RouteMessage::new_route(Destination::Network(destination));
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
