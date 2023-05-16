use std::{
    ffi::CString,
    net::{Ipv4Addr, Ipv6Addr},
};

use ipnetwork::IpNetwork;
use nix::net::if_::if_nametoindex;
use system_configuration::{
    core_foundation::string::CFString,
    network_configuration::{SCNetworkService, SCNetworkSet},
    preferences::SCPreferences,
};

use super::watch::{
    data::{Destination, RouteMessage},
    RoutingTable,
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
        Family::V4 => v4_default(),
        Family::V6 => v6_default(),
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
            Err(error) => {
                log::error!("Failed to get index of network interface: {error}");
                continue;
            }
        };

        // Request ifscoped route for this interface
        let route_msg = msg.clone().set_ifscope(u16::try_from(index).unwrap());
        if let Ok(Some(route)) = routing_table.get_route(&route_msg).await {
            return Some(route);
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

fn v4_default() -> IpNetwork {
    IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap()
}

fn v6_default() -> IpNetwork {
    IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap()
}
