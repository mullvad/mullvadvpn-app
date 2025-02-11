#![allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use ipnetwork::IpNetwork;
use nix::{
    net::if_::if_nametoindex,
    sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage},
};
use std::{
    collections::BTreeMap,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use super::data::{Destination, RouteMessage};
use system_configuration::{
    core_foundation::{
        array::CFArray,
        base::{CFType, TCFType, ToVoid},
        dictionary::CFDictionary,
        runloop::{kCFRunLoopCommonModes, CFRunLoop},
        string::{CFString, CFStringRef},
    },
    dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder, SCDynamicStoreCallBackContext},
    network_configuration::SCNetworkSet,
    preferences::SCPreferences,
    sys::schema_definitions::{
        kSCDynamicStorePropNetPrimaryInterface, kSCPropInterfaceName, kSCPropNetIPv4Addresses,
        kSCPropNetIPv4Router, kSCPropNetIPv6Addresses, kSCPropNetIPv6Router,
    },
};

const STATE_IPV4_KEY: &str = "State:/Network/Global/IPv4";
const STATE_IPV6_KEY: &str = "State:/Network/Global/IPv6";
const STATE_SERVICE_PATTERN: &str = "State:/Network/Service/.*/IP.*";

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

impl Family {
    pub fn default_network(self) -> IpNetwork {
        match self {
            Family::V4 => IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap(),
            Family::V6 => IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap(),
        }
    }
}

#[derive(Debug)]
struct NetworkServiceDetails {
    name: String,
    router_ip: IpAddr,
    first_ip: IpAddr,
}

pub struct PrimaryInterfaceMonitor {
    store: SCDynamicStore,
    prefs: SCPreferences,
}

// FIXME: Implement Send on SCDynamicStore, if it's safe
unsafe impl Send for PrimaryInterfaceMonitor {}

pub enum InterfaceEvent {
    Update,
}

/// Default interface/route
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultRoute {
    /// Default interface name
    pub interface: String,
    /// Default interface index
    pub interface_index: u16,
    /// Router IP
    pub router_ip: IpAddr,
    /// Default interface IP address
    pub ip: IpAddr,
}

impl From<DefaultRoute> for RouteMessage {
    fn from(route: DefaultRoute) -> Self {
        let network = if route.router_ip.is_ipv4() {
            Family::V4.default_network()
        } else {
            Family::V6.default_network()
        };
        // The route message requires a socket address. The port is ignored in this case.
        let router_addr = SocketAddr::from((route.router_ip, 0));
        RouteMessage::new_route(Destination::Network(network))
            .set_gateway_addr(router_addr)
            .set_interface_index(route.interface_index)
    }
}

impl PrimaryInterfaceMonitor {
    pub fn new() -> (Self, UnboundedReceiver<InterfaceEvent>) {
        let store = SCDynamicStoreBuilder::new("talpid-routing").build();
        let prefs = SCPreferences::default(&CFString::new("talpid-routing"));

        let (tx, rx) = mpsc::unbounded();
        Self::start_listener(tx);

        (Self { store, prefs }, rx)
    }

    fn start_listener(tx: UnboundedSender<InterfaceEvent>) {
        std::thread::spawn(|| {
            let listener_store = SCDynamicStoreBuilder::new("talpid-routing-listener")
                .callback_context(SCDynamicStoreCallBackContext {
                    callout: Self::store_change_handler,
                    info: tx,
                })
                .build();

            let watch_keys: CFArray<CFString> = CFArray::from_CFTypes(&[
                CFString::new(STATE_IPV4_KEY),
                CFString::new(STATE_IPV6_KEY),
            ]);
            let watch_patterns = CFArray::from_CFTypes(&[CFString::new(STATE_SERVICE_PATTERN)]);

            if !listener_store.set_notification_keys(&watch_keys, &watch_patterns) {
                log::error!("Failed to start interface listener");
                return;
            }

            let run_loop_source = listener_store.create_run_loop_source();
            CFRunLoop::get_current().add_source(&run_loop_source, unsafe { kCFRunLoopCommonModes });
            CFRunLoop::run_current();

            log::debug!("Interface listener exiting");
        });
    }

    fn store_change_handler(
        _store: SCDynamicStore,
        changed_keys: CFArray<CFString>,
        tx: &mut UnboundedSender<InterfaceEvent>,
    ) {
        for k in changed_keys.iter() {
            log::debug!("Interface change, key {}", k.to_string());
        }
        let _ = tx.unbounded_send(InterfaceEvent::Update);
    }

    /// Retrieve the best current default route. This is based on the primary interface, or else
    /// the first active interface in the network service order.
    pub fn get_route(&self, family: Family) -> Option<DefaultRoute> {
        let ifaces = self
            .get_primary_interface(family)
            .map(|iface| {
                log::debug!("Found primary interface for {family}");
                vec![iface]
            })
            .unwrap_or_else(|| self.network_services(family));

        let (iface, index) = ifaces
            .into_iter()
            .filter_map(|iface| {
                let index = if_nametoindex(iface.name.as_str())
                    .inspect_err(|error| {
                        log::error!(
                            "Failed to retrieve interface index for \"{}\": {error}",
                            iface.name
                        );
                    })
                    .ok()?;
                Some((iface, index))
            })
            .next()?;

        let index = u16::try_from(index).unwrap();

        let mut router_ip = iface.router_ip;
        if let IpAddr::V6(ref mut addr) = router_ip {
            if is_link_local_v6(addr) {
                // The second pair of octets should be set to the scope id
                // See getaddr() in route.c:
                // https://opensource.apple.com/source/network_cmds/network_cmds-396.6/route.tproj/route.c.auto.html

                let second_octet = index.to_be_bytes();

                let mut octets = addr.octets();
                octets[2] = second_octet[0];
                octets[3] = second_octet[1];

                *addr = Ipv6Addr::from(octets);
            }
        }

        Some(DefaultRoute {
            interface: iface.name,
            interface_index: index,
            router_ip,
            ip: iface.first_ip,
        })
    }

    fn get_primary_interface(&self, family: Family) -> Option<NetworkServiceDetails> {
        let key = if family == Family::V4 {
            STATE_IPV4_KEY
        } else {
            STATE_IPV6_KEY
        };
        let global_dict = self
            .store
            .get(key)
            .and_then(|v| v.downcast_into::<CFDictionary>())?;
        let name = get_dict_elem_as_string(&global_dict, unsafe {
            kSCDynamicStorePropNetPrimaryInterface
        })
        .or_else(|| {
            log::debug!("Missing name for primary interface ({family})");
            None
        })?;
        let router_ip = get_service_router_ip(&global_dict, family).or_else(|| {
            log::debug!("Missing router IP for primary interface ({name}, {family})");
            None
        })?;
        let first_ip = get_service_first_ip(&global_dict, family).or_else(|| {
            log::debug!("Missing IP for primary interface ({name}, {family})");
            None
        })?;
        Some(NetworkServiceDetails {
            name,
            router_ip,
            first_ip,
        })
    }

    fn network_services(&self, family: Family) -> Vec<NetworkServiceDetails> {
        SCNetworkSet::new(&self.prefs)
            .service_order()
            .iter()
            .filter_map(|service_id| {
                let service_id_s = service_id.to_string();
                let service_key = if family == Family::V4 {
                    format!("State:/Network/Service/{service_id_s}/IPv4")
                } else {
                    format!("State:/Network/Service/{service_id_s}/IPv6")
                };
                let service_dict = self
                    .store
                    .get(CFString::new(&service_key))
                    .and_then(|v| v.downcast_into::<CFDictionary>())?;
                let name = get_dict_elem_as_string(&service_dict, unsafe { kSCPropInterfaceName })
                    .or_else(|| {
                        log::debug!("Missing name for service {service_key} ({family})");
                        None
                    })?;
                let router_ip = get_service_router_ip(&service_dict, family).or_else(|| {
                    log::debug!("Missing router IP for {service_key} ({name}, {family})");
                    None
                })?;
                let first_ip = get_service_first_ip(&service_dict, family).or_else(|| {
                    log::debug!("Missing IP for \"{service_key}\" ({name}, {family})");
                    None
                })?;
                Some(NetworkServiceDetails {
                    name,
                    router_ip,
                    first_ip,
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn debug(&self) {
        for family in [Family::V4, Family::V6] {
            log::debug!(
                "Primary interface ({family}): {:?}",
                self.get_primary_interface(family)
            );
            log::debug!(
                "Network services ({family}): {:?}",
                self.network_services(family)
            );
        }
    }
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

fn is_link_local_v6(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

fn get_service_router_ip(service_dict: &CFDictionary, family: Family) -> Option<IpAddr> {
    let router_key = if family == Family::V4 {
        unsafe { kSCPropNetIPv4Router }
    } else {
        unsafe { kSCPropNetIPv6Router }
    };
    get_dict_elem_as_string(service_dict, router_key).and_then(|ip| ip.parse().ok())
}

/// Return the first IP address of a network service (e.g., a dictionary at
/// `State:/Network/Service/{service id}/IPv4`).
/// The array of IP addresses is found using the key `kSCPropNetIPv4Addresses` or
/// `kSCPropNetIPv6Addresses`, depending on the family.
fn get_service_first_ip(service_dict: &CFDictionary, family: Family) -> Option<IpAddr> {
    let ip_key = if family == Family::V4 {
        unsafe { kSCPropNetIPv4Addresses }
    } else {
        unsafe { kSCPropNetIPv6Addresses }
    };
    service_dict
        .find(ip_key.to_void())
        .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
        .and_then(|s| s.downcast::<CFArray>())
        .and_then(|ips| {
            ips.get(0)
                .map(|ip| unsafe { CFType::wrap_under_get_rule(*ip) })
        })
        .and_then(|s| s.downcast::<CFString>())
        .map(|s| s.to_string())
        .and_then(|ip| ip.parse().ok())
}

fn get_dict_elem_as_string(dict: &CFDictionary, key: CFStringRef) -> Option<String> {
    dict.find(key.to_void())
        .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
        .and_then(|s| s.downcast::<CFString>())
        .map(|s| s.to_string())
}
