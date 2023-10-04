use ipnetwork::IpNetwork;
use nix::{
    net::if_::{if_nametoindex, InterfaceFlags},
    sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage},
};
use std::{
    collections::BTreeMap,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use system_configuration::{
    core_foundation::{
        base::{CFType, TCFType, ToVoid},
        dictionary::CFDictionary,
        string::CFString,
    },
    dynamic_store::SCDynamicStoreBuilder,
    network_configuration::SCNetworkSet,
    preferences::SCPreferences,
    sys::schema_definitions::{kSCPropInterfaceName, kSCPropNetIPv4Router},
};

use super::data::{Destination, RouteMessage};

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

/// Retrieve the best current default route. That is the first scoped default route, ordered by
/// network service order, and with interfaces filtered out if they do not have valid IP addresses
/// assigned.
///
/// # Note
///
/// The tunnel interface is not even listed in the service order, so it will be skipped.
pub async fn get_best_default_route(family: Family) -> Option<RouteMessage> {
    for iface in network_service_order(family) {
        let Ok(index) = if_nametoindex(iface.name.as_str()) else {
            continue;
        };

        // Request ifscoped default route for this interface
        let msg = RouteMessage::new_route(Destination::Network(IpNetwork::from(family)))
            .set_gateway_addr(iface.router_ip)
            .set_interface_index(u16::try_from(index).unwrap());
        if is_active_interface(&iface.name, family).unwrap_or(true) {
            return Some(msg);
        }
    }

    None
}

/// Return a map from interface name to link addresses (AF_LINK)
pub fn get_interface_link_addresses() -> io::Result<BTreeMap<String, SockaddrStorage>> {
    let mut gateway_link_addrs = BTreeMap::new();
    let addrs = nix::ifaddrs::getifaddrs()?;
    for addr in addrs.into_iter() {
        if addr.address.and_then(|addr| addr.family()) != Some(AddressFamily::Link) {
            continue;
        }
        gateway_link_addrs.insert(addr.interface_name, addr.address.unwrap());
    }
    Ok(gateway_link_addrs)
}

struct NetworkServiceDetails {
    name: String,
    router_ip: IpAddr,
}

fn network_service_order(family: Family) -> Vec<NetworkServiceDetails> {
    let prefs = SCPreferences::default(&CFString::new("talpid-routing"));
    let set = SCNetworkSet::new(&prefs);
    let service_order = set.service_order();
    let store = SCDynamicStoreBuilder::new("mullvad-routing").build();

    service_order
        .iter()
        .filter_map(|service_id| {
            let service_id_s = service_id.to_string();
            let key = if family == Family::V4 {
                format!("State:/Network/Service/{service_id_s}/IPv4")
            } else {
                format!("State:/Network/Service/{service_id_s}/IPv6")
            };

            let ip_dict = store
                .get(CFString::new(&key))
                .and_then(|v| v.downcast_into::<CFDictionary>())?;
            let name = ip_dict
                .find(unsafe { kSCPropInterfaceName }.to_void())
                .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
                .and_then(|s| s.downcast::<CFString>())
                .map(|s| s.to_string())?;
            let router_ip = ip_dict
                .find(unsafe { kSCPropNetIPv4Router }.to_void())
                .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
                .and_then(|s| s.downcast::<CFString>())
                .and_then(|ip| ip.to_string().parse().ok())?;

            Some(NetworkServiceDetails { name, router_ip })
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
