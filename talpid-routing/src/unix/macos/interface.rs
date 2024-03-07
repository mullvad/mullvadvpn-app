use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use ipnetwork::IpNetwork;
use nix::{
    net::if_::{if_nametoindex, InterfaceFlags},
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
        string::CFString,
    },
    dynamic_store::{SCDynamicStore, SCDynamicStoreBuilder, SCDynamicStoreCallBackContext},
    network_configuration::SCNetworkSet,
    preferences::SCPreferences,
    sys::schema_definitions::{
        kSCDynamicStorePropNetPrimaryInterface, kSCPropInterfaceName, kSCPropNetIPv4Router,
        kSCPropNetIPv6Router,
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
    pub fn get_route(&self, family: Family) -> Option<RouteMessage> {
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
                let index = if_nametoindex(iface.name.as_str()).map_err(|error| {
                    log::error!("Failed to retrieve interface index for \"{}\": {error}", iface.name);
                    error
                }).ok()?;

                let active = is_active_interface(&iface.name, family).unwrap_or_else(|error| {
                    log::error!("is_active_interface() returned an error for interface \"{}\", assuming active. Error: {error}", iface.name);
                    true
                });
                if !active {
                    log::debug!("Skipping inactive interface {}, router IP {}", iface.name, iface.router_ip);
                    return None;
                }
                Some((iface, index))
            })
            .next()?;

        let router_addr = (iface.router_ip, 0);
        let mut router_addr = SocketAddr::from(router_addr);

        // If the gateway is a link-local address, scope ID must be specified
        if let SocketAddr::V6(ref mut v6_addr) = router_addr {
            let v6ip = v6_addr.ip();

            if is_link_local_v6(v6ip) {
                // The second pair of octets should be set to the scope id
                // See getaddr() in route.c:
                // https://opensource.apple.com/source/network_cmds/network_cmds-396.6/route.tproj/route.c.auto.html

                let second_octet = u16::try_from(index).unwrap().to_be_bytes();

                let mut octets = v6ip.octets();
                octets[2] = second_octet[0];
                octets[3] = second_octet[1];

                let new_ip = Ipv6Addr::from(octets);

                v6_addr.set_ip(new_ip);
            }
        }

        let msg = RouteMessage::new_route(Destination::Network(family.default_network()))
            .set_gateway_addr(router_addr)
            .set_interface_index(u16::try_from(index).unwrap());
        Some(msg)
    }

    fn get_primary_interface(&self, family: Family) -> Option<NetworkServiceDetails> {
        let global_name = if family == Family::V4 {
            STATE_IPV4_KEY
        } else {
            STATE_IPV6_KEY
        };
        let global_dict = self
            .store
            .get(CFString::new(global_name))
            .and_then(|v| v.downcast_into::<CFDictionary>())?;
        let name = global_dict
            .find(unsafe { kSCDynamicStorePropNetPrimaryInterface }.to_void())
            .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
            .and_then(|s| s.downcast::<CFString>())
            .map(|s| s.to_string())
            .or_else(|| {
                log::debug!("Missing name for primary interface ({family})");
                None
            })?;

        let router_key = if family == Family::V4 {
            unsafe { kSCPropNetIPv4Router.to_void() }
        } else {
            unsafe { kSCPropNetIPv6Router.to_void() }
        };

        let router_ip = global_dict
            .find(router_key)
            .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
            .and_then(|s| s.downcast::<CFString>())
            .and_then(|ip| ip.to_string().parse().ok())
            .or_else(|| {
                log::debug!("Missing router IP for primary interface \"{name}\"");
                None
            })?;

        Some(NetworkServiceDetails { name, router_ip })
    }

    fn network_services(&self, family: Family) -> Vec<NetworkServiceDetails> {
        let router_key = if family == Family::V4 {
            unsafe { kSCPropNetIPv4Router.to_void() }
        } else {
            unsafe { kSCPropNetIPv6Router.to_void() }
        };

        SCNetworkSet::new(&self.prefs)
            .service_order()
            .iter()
            .filter_map(|service_id| {
                let service_id_s = service_id.to_string();
                let key = if family == Family::V4 {
                    format!("State:/Network/Service/{service_id_s}/IPv4")
                } else {
                    format!("State:/Network/Service/{service_id_s}/IPv6")
                };

                let ip_dict = self
                    .store
                    .get(CFString::new(&key))
                    .and_then(|v| v.downcast_into::<CFDictionary>())?;
                let name = ip_dict
                    .find(unsafe { kSCPropInterfaceName }.to_void())
                    .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
                    .and_then(|s| s.downcast::<CFString>())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        log::debug!("Missing name for service {service_id_s} ({family})");
                        None
                    })?;
                let router_ip = ip_dict
                    .find(router_key)
                    .map(|s| unsafe { CFType::wrap_under_get_rule(*s) })
                    .and_then(|s| s.downcast::<CFString>())
                    .and_then(|ip| ip.to_string().parse().ok())
                    .or_else(|| {
                        log::debug!("Missing router IP for {service_id_s} ({name}, {family})");
                        None
                    })?;

                Some(NetworkServiceDetails { name, router_ip })
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
                    Family::V4 => matches!(addr.as_sockaddr_in(), Some(addr_in) if is_routable_v4(&addr_in.ip())),
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
    !addr.is_unspecified() && !addr.is_loopback() && !is_link_local_v6(addr)
}

fn is_link_local_v6(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}
