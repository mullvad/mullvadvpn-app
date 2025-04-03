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
    ops::Deref,
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
};

const STATE_IPV4_KEY: &str = "State:/Network/Global/IPv4";
const STATE_IPV6_KEY: &str = "State:/Network/Global/IPv6";
const STATE_SERVICE_PATTERN: &str = "State:/Network/Service/.*/IP.*";

/// Safely read a symbol in [system_configuration::sys::schema_definitions].
macro_rules! schema_definition {
    ($name:ident) => {
        // SAFETY: system_configuration_sys is generated using bindgen, and all symbols in the
        // schema_definitions module are to static string pointers, and should be safe to read.
        unsafe { ::system_configuration::sys::schema_definitions::$name }
    };
}

// TODO: replace with IpVersion from talpid-types?
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

pub struct PrimaryInterfaceMonitor {
    store: SCDynamicStore,
    prefs: SCPreferences,
}

// FIXME: Implement Send on SCDynamicStore, if it's safe
unsafe impl Send for PrimaryInterfaceMonitor {}

/// Contents of a `/Network/Service/<service_id>/IPvX` key in the [SCDynamicStore].
#[derive(Clone, Debug)]
pub struct NetworkServiceDetails {
    pub interface_name: String,
    pub router_ip: IpAddr,
    pub first_ip: IpAddr,
}

/// Contents of the `/Network/Global/IPvX` key in the [SCDynamicStore].
#[derive(Clone, Debug)]
pub struct PrimaryInterfaceDetails {
    pub name: String,
    pub service_id: String,
}

pub enum InterfaceEvent {
    /// The `/Network/Global/IPvX` key in the [SCDynamicStore] was updated.
    PrimaryInterfaceUpdate {
        /// The IP address family.
        family: Family,

        /// The updated [PrimaryInterfaceDetails].
        new_value: Option<PrimaryInterfaceDetails>,
    },

    /// A network service in the [SCDynamicStore] was updated.
    NetworkServiceUpdate {
        /// The IP address family of the network service.
        family: Family,

        /// The ID of the network service.
        service_id: String,

        /// The updated [NetworkServiceDetails].
        new_value: Option<NetworkServiceDetails>,
    },
}
impl InterfaceEvent {
    pub fn family(&self) -> Family {
        match *self {
            InterfaceEvent::PrimaryInterfaceUpdate { family, .. } => family,
            InterfaceEvent::NetworkServiceUpdate { family, .. } => family,
        }
    }
}

/// The best network route. Either suggested by macOS, or inferred by looking at the available
/// network interfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultRoute {
    /// Interface name.
    pub interface: String,

    /// Interface index.
    pub interface_index: u16,

    /// IP address of the interface.
    pub ip: IpAddr,

    /// Router IP.
    pub router_ip: IpAddr,
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
    pub fn new() -> (Self, UnboundedReceiver<Vec<InterfaceEvent>>) {
        let store = SCDynamicStoreBuilder::new("talpid-routing").build();
        let prefs = SCPreferences::default(&CFString::new("talpid-routing"));

        let (tx, rx) = mpsc::unbounded();
        Self::start_listener(tx);

        (Self { store, prefs }, rx)
    }

    fn start_listener(tx: UnboundedSender<Vec<InterfaceEvent>>) {
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

            // SAFETY: this is just a static string pointer, referencing it should be safe.
            let run_loop_common_modes = unsafe { kCFRunLoopCommonModes };

            CFRunLoop::get_current().add_source(&run_loop_source, run_loop_common_modes);
            CFRunLoop::run_current();

            log::debug!("Interface listener exiting");
        });
    }

    fn store_change_handler(
        store: SCDynamicStore,
        changed_keys: CFArray<CFString>,
        tx: &mut UnboundedSender<Vec<InterfaceEvent>>,
    ) {
        let events = changed_keys
            .iter()
            .filter_map(|key| {
                let key = key.deref().to_string();

                let family = match key.as_str() {
                    STATE_IPV4_KEY => Family::V4,
                    STATE_IPV6_KEY => Family::V6,

                    key => {
                        let Some((service_id, family)) = service_id_from_service_key(key) else {
                            log::debug!("Unknown SCDynStore key: {key:?}");
                            return None; // skip invalid keys
                        };

                        // TODO: distinguish between errors and None?
                        let new_value = get_network_service(&store, service_id, family);
                        return Some(InterfaceEvent::NetworkServiceUpdate {
                            family,
                            service_id: service_id.to_string(),
                            new_value,
                        });
                    }
                };

                // TODO: distinguish between errors and None?
                let new_value = get_primary_interface(&store, family);
                Some(InterfaceEvent::PrimaryInterfaceUpdate { family, new_value })
            })
            .collect();

        let _ = tx.unbounded_send(events);
    }

    /// Retrieve the best current default route. This is based on the primary interface, or else
    /// the first active interface in the network service order.
    pub fn get_route(&self, family: Family) -> Option<DefaultRoute> {
        self.get_primary_interface_service(family)
            .map(|service| {
                log::debug!("Found primary interface for {family}");
                vec![service]
            })
            .unwrap_or_else(|| self.network_services(family))
            .into_iter()
            .filter_map(|service| self.route_from_service(&service))
            .next()
    }

    /// Iterate through active interfaces in network service order and return a suggested route for
    /// the first one with a valid IP and gateway.
    pub fn get_route_by_service_order(&self, family: Family) -> Option<DefaultRoute> {
        self.network_services(family)
            .into_iter()
            .filter_map(|service| self.route_from_service(&service))
            .next()
    }

    pub fn route_from_service(&self, service: &NetworkServiceDetails) -> Option<DefaultRoute> {
        let index = if_nametoindex(service.interface_name.as_str())
            .inspect_err(|error| {
                log::error!(
                    "Failed to retrieve interface index for \"{}\": {error}",
                    service.interface_name
                );
            })
            .ok()?;

        let index = u16::try_from(index).unwrap();

        let mut router_ip = service.router_ip;
        if let IpAddr::V6(addr) = &mut router_ip {
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
            interface: service.interface_name.clone(),
            interface_index: index,
            router_ip,
            ip: service.first_ip,
        })
    }

    fn get_primary_interface_service(&self, family: Family) -> Option<NetworkServiceDetails> {
        get_primary_interface_service(&self.store, family)
    }

    pub fn get_network_service(
        &self,
        service_id: &str,
        family: Family,
    ) -> Option<NetworkServiceDetails> {
        get_network_service(&self.store, service_id, family)
    }

    fn network_services(&self, family: Family) -> Vec<NetworkServiceDetails> {
        SCNetworkSet::new(&self.prefs)
            .service_order()
            .iter()
            .filter_map(|service_id| self.get_network_service(&service_id.to_string(), family))
            .collect::<Vec<_>>()
    }

    pub fn debug(&self) {
        for family in [Family::V4, Family::V6] {
            log::debug!(
                "Primary interface ({family}): {:?}",
                self.get_primary_interface_service(family)
            );
            log::debug!(
                "Network services ({family}): {:?}",
                self.network_services(family)
            );
        }
    }
}

/// Construct the string key for a network service from its ID.
fn network_service_key(service_id: String, family: Family) -> String {
    let family = match family {
        Family::V4 => "IPv4",
        Family::V6 => "IPv6",
    };

    format!("State:/Network/Service/{service_id}/{family}")
}

fn service_id_from_service_key(key: &str) -> Option<(&str, Family)> {
    let id_and_family = key.strip_prefix("State:/Network/Service/")?;
    let (id, family) = id_and_family.split_once('/')?;
    let family = match family {
        "IPv4" => Family::V4,
        "IPv6" => Family::V6,
        _ => return None,
    };

    Some((id, family))
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

fn get_primary_interface(
    store: &SCDynamicStore,
    family: Family,
) -> Option<PrimaryInterfaceDetails> {
    let key = if family == Family::V4 {
        STATE_IPV4_KEY
    } else {
        STATE_IPV6_KEY
    };
    let global_dict = store
        .get(key)
        .or_else(|| {
            log::debug!("{key} is missing!");
            None
        })
        .and_then(|v| v.downcast_into::<CFDictionary>())?;

    let service_id = get_dict_elem_as_string(
        &global_dict,
        schema_definition!(kSCDynamicStorePropNetPrimaryService),
    )
    .or_else(|| {
        log::debug!("Missing service ID for primary interface ({family})");
        None
    })?;

    let name = get_dict_elem_as_string(
        &global_dict,
        schema_definition!(kSCDynamicStorePropNetPrimaryInterface),
    )
    .or_else(|| {
        log::debug!("Missing name for primary interface ({family})");
        None
    })?;

    Some(PrimaryInterfaceDetails { name, service_id })
}

fn get_primary_interface_service(
    store: &SCDynamicStore,
    family: Family,
) -> Option<NetworkServiceDetails> {
    let primary_interface = get_primary_interface(store, family)?;
    get_network_service(store, &primary_interface.service_id, family)
}

/// Get details about a specific network interface.
///
/// Will return `None` and log a message on any error.
fn get_network_service(
    store: &SCDynamicStore,
    service_id: &str,
    family: Family,
) -> Option<NetworkServiceDetails> {
    let service_key = network_service_key(service_id.to_string(), family);
    let service_dict = store
        .get(CFString::new(&service_key))
        .and_then(|v| v.downcast_into::<CFDictionary>())?;

    let interface_name =
        get_dict_elem_as_string(&service_dict, schema_definition!(kSCPropInterfaceName)).or_else(
            || {
                log::debug!("Missing name for service {service_key} ({family})");
                None
            },
        )?;
    let router_ip = get_service_router_ip(&service_dict, family).or_else(|| {
        log::debug!("Missing router IP for {service_key} ({interface_name}, {family})");
        log::debug!("{service_key} {service_dict:?}");
        None
    })?;
    let first_ip = get_service_first_ip(&service_dict, family).or_else(|| {
        log::debug!("Missing IP for \"{service_key}\" ({interface_name}, {family})");
        None
    })?;

    Some(NetworkServiceDetails {
        interface_name,
        router_ip,
        first_ip,
    })
}

fn get_service_router_ip(service_dict: &CFDictionary, family: Family) -> Option<IpAddr> {
    let router_key = if family == Family::V4 {
        schema_definition!(kSCPropNetIPv4Router)
    } else {
        schema_definition!(kSCPropNetIPv6Router)
    };
    get_dict_elem_as_string(service_dict, router_key).and_then(|ip| ip.parse().ok())
}

/// Return the first IP address of a network service (e.g., a dictionary at
/// `State:/Network/Service/{service id}/IPv4`).
/// The array of IP addresses is found using the key `kSCPropNetIPv4Addresses` or
/// `kSCPropNetIPv6Addresses`, depending on the family.
fn get_service_first_ip(service_dict: &CFDictionary, family: Family) -> Option<IpAddr> {
    let ip_key = if family == Family::V4 {
        schema_definition!(kSCPropNetIPv4Addresses)
    } else {
        schema_definition!(kSCPropNetIPv6Addresses)
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
