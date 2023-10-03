use ipnetwork::IpNetwork;
use nix::{
    net::if_::{if_nametoindex, InterfaceFlags},
    sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage},
};
use std::{
    collections::BTreeMap,
    ffi::CString,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use system_configuration::{
    core_foundation::string::CFString,
    network_configuration::{SCNetworkService, SCNetworkSet},
    preferences::SCPreferences,
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
    let mut msg = RouteMessage::new_route(Destination::Network(IpNetwork::from(family)));
    msg = msg.set_gateway_route(true);

    for iface in network_service_order() {
        let Ok(Some(router_addr)) = get_router_address(family, &iface).await else {
            continue;
        };

        let iface_bytes = match CString::new(iface.as_bytes()) {
            Ok(name) => name,
            Err(error) => {
                log::error!("Invalid interface name: {iface}, {error}");
                continue;
            }
        };

        // Get interface ID
        let Ok(index) = if_nametoindex(iface_bytes.as_c_str()) else {
            continue;
        };

        // Request ifscoped default route for this interface
        let route_msg = msg
            .clone()
            .set_gateway_addr(router_addr)
            .set_interface_index(u16::try_from(index).unwrap());
        if is_active_interface(&iface, family).unwrap_or(true) {
            return Some(route_msg);
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

async fn get_router_address(family: Family, interface_name: &str) -> io::Result<Option<IpAddr>> {
    let output = tokio::process::Command::new("ipconfig")
        .arg("getsummary")
        .arg(interface_name)
        .output()
        .await?
        .stdout;

    let Ok(output_str) = std::str::from_utf8(&output) else {
        return Ok(None);
    };

    match family {
        Family::V4 => Ok(parse_v4_ipconfig_output(output_str)),
        Family::V6 => Ok(parse_v6_ipconfig_output(output_str)),
    }
}

fn parse_v4_ipconfig_output(output: &str) -> Option<IpAddr> {
    let mut iter = output.split_whitespace();
    loop {
        let next_chunk = iter.next()?;
        if next_chunk == "Router" && iter.next()? == ":" {
            return iter.next()?.parse().ok();
        }
    }
}

fn parse_v6_ipconfig_output(output: &str) -> Option<IpAddr> {
    let mut iter = output.split_whitespace();
    let pattern = ["RouterAdvertisement", ":", "from"];
    'outer: loop {
        let mut next_chunk = iter.next()?;
        for expected_chunk in pattern {
            if expected_chunk != next_chunk {
                continue 'outer;
            }
            next_chunk = iter.next()?;
        }
        return next_chunk.trim_end_matches(",").parse().ok();
    }
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

#[cfg(test)]
const TEST_IPCONFIG_OUTPUT: &str = "<dictionary> {
  Hashed-BSSID : 86:a2:7a:bb:7c:5c
  IPv4 : <array> {
    0 : <dictionary> {
      Addresses : <array> {
        0 : 192.168.1.3
      }
      ChildServiceID : LINKLOCAL-en0
      ConfigMethod : Manual
      IsPublished : TRUE
      ManualAddress : 192.168.1.3
      ManualSubnetMask : 255.255.255.0
      Router : 192.168.1.1
      RouterARPVerified : TRUE
      ServiceID : 400B48FB-2585-41DF-8459-30C5C6D5621C
      SubnetMasks : <array> {
        0 : 255.255.255.0
      }
    }
    1 : <dictionary> {
      ConfigMethod : LinkLocal
      IsPublished : TRUE
      ParentServiceID : 400B48FB-2585-41DF-8459-30C5C6D5621C
      ServiceID : LINKLOCAL-en0
    }
  }
  IPv6 : <array> {
    0 : <dictionary> {
      ConfigMethod : Automatic
      DHCPv6 : <dictionary> {
        ElapsedTime : 2200
        Mode : Stateful
        State : Solicit
      }
      IsPublished : TRUE
      RTADV : <dictionary> {
        RouterAdvertisement : from fe80::5aef:68ff:fe0d:18db, length 88, hop limit 0, lifetime 1800s, reacha
ble 0ms, retransmit 0ms, flags 0xc4=[ managed other proxy ], pref=medium
        source link-address option (1), length 8 (1): 58:ef:68:0d:18:db
        prefix info option (3), length 32 (4):  ::/64, Flags [ onlink ], valid time 2592000s, pref. time 604
800s
        prefix info option (3), length 32 (4):  2a03:1b20:5:7::/64, Flags [ onlink auto ], valid time 259200
0s, pref. time 604800s

        State : Acquired
      }
      ServiceID : 400B48FB-2585-41DF-8459-30C5C6D5621C
    }
  }
  InterfaceType : WiFi
  LinkStatusActive : TRUE
  NetworkID : 350BCC68-6D65-4D4A-9187-264D7B543738
  SSID : app-team-lab
  Security : WPA2_PSK
}";

#[test]
fn test_parsing_v4_ipconfig_output() {
    assert_eq!(
        parse_v4_ipconfig_output(&TEST_IPCONFIG_OUTPUT).unwrap(),
        "192.168.1.1".parse::<IpAddr>().unwrap()
    )
}

#[test]
fn test_parsing_v6_ipconfig_output() {
    assert_eq!(
        parse_v6_ipconfig_output(&TEST_IPCONFIG_OUTPUT).unwrap(),
        "fe80::5aef:68ff:fe0d:18db".parse::<IpAddr>().unwrap()
    )
}
