use super::{
    default_route_monitor::{DefaultRouteMonitor, EventType as RouteMonitorEventType},
    get_best_default_route, Error, InterfaceAndGateway, Result,
};
use crate::NetNode;
use ipnetwork::IpNetwork;
use std::{
    collections::HashMap,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
};
use talpid_windows_net::{
    inet_sockaddr_from_socketaddr, try_socketaddr_from_inet_sockaddr, AddressFamily,
};
use widestring::{WideCStr, WideCString};
use windows_sys::Win32::{
    Foundation::{
        ERROR_BUFFER_OVERFLOW, ERROR_NOT_FOUND, ERROR_NO_DATA, ERROR_OBJECT_ALREADY_EXISTS,
        ERROR_SUCCESS, NO_ERROR,
    },
    NetworkManagement::{
        IpHelper::{
            ConvertInterfaceAliasToLuid, CreateIpForwardEntry2, DeleteIpForwardEntry2,
            GetAdaptersAddresses, InitializeIpForwardEntry, SetIpForwardEntry2,
            GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
            GAA_FLAG_SKIP_FRIENDLY_NAME, GAA_FLAG_SKIP_MULTICAST, GET_ADAPTERS_ADDRESSES_FLAGS,
            IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_GATEWAY_ADDRESS_LH, IP_ADAPTER_IPV4_ENABLED,
            IP_ADAPTER_IPV6_ENABLED, IP_ADDRESS_PREFIX, MIB_IPFORWARD_ROW2,
        },
        Ndis::NET_LUID_LH,
    },
    Networking::WinSock::{
        NlroManual, ADDRESS_FAMILY, AF_INET, AF_INET6, MIB_IPPROTO_NETMGMT, SOCKADDR_IN,
        SOCKADDR_IN6, SOCKADDR_INET, SOCKET_ADDRESS,
    },
};

type Network = IpNetwork;
type NodeAddress = SOCKADDR_INET;

/// Callback handle for the default route changed callback. Produced by the RouteManager.
pub struct CallbackHandle {
    nonce: i32,
    callbacks: Arc<Mutex<(i32, HashMap<i32, Callback>)>>,
}

impl Drop for CallbackHandle {
    fn drop(&mut self) {
        let (_, callbacks) = &mut *self.callbacks.lock().unwrap();
        match callbacks.remove(&self.nonce) {
            Some(_) => (),
            None => {
                log::warn!("Could not un-register route manager callback due to it already being de-registered");
            }
        }
    }
}

#[derive(Clone)]
struct RegisteredRoute {
    network: Network,
    luid: NET_LUID_LH,
    next_hop: SocketAddr,
}

impl std::fmt::Display for RegisteredRoute {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: luid.Value is always valid as the underlying type of both union fields is an u64
        formatter.write_fmt(format_args!("RegisteredRoute {{ luid: {} }}", unsafe {
            self.luid.Value
        }))
    }
}

impl PartialEq for RegisteredRoute {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: luid.Value is always valid as the underlying type of both union fields is an u64
        (unsafe { self.luid.Value == other.luid.Value })
            && (self.next_hop == other.next_hop)
            && (self.network == other.network)
    }
}

#[derive(Clone)]
pub struct Node {
    pub device_name: Option<widestring::U16CString>,
    pub gateway: Option<NodeAddress>,
}

#[derive(Clone)]
pub struct Route {
    pub network: Network,
    pub node: NetNode,
}

#[derive(Clone)]
struct RouteRecord {
    route: Route,
    registered_route: RegisteredRoute,
}

struct EventEntry {
    record: RouteRecord,
    event_type: RecordEventType,
}

enum RecordEventType {
    AddRoute,
}

pub type Callback = Box<dyn for<'a> Fn(RouteMonitorEventType<'a>, AddressFamily) + Send>;

pub struct RouteManagerInternal {
    route_monitor_v4: Option<DefaultRouteMonitor>,
    route_monitor_v6: Option<DefaultRouteMonitor>,
    routes: Arc<Mutex<Vec<RouteRecord>>>,
    /// Lock for a nonce and a HashMap of callbacks and their id which is used as a handle to
    /// unregister them. The nonce is used to create new ids and then incrementing.
    callbacks: Arc<Mutex<(i32, HashMap<i32, Callback>)>>,
}

impl RouteManagerInternal {
    pub fn new() -> Result<Self> {
        let routes = Arc::new(Mutex::new(Vec::new()));
        let callbacks = Arc::new(Mutex::new((0, HashMap::new())));

        let callbacks_ipv4 = callbacks.clone();
        let routes_ipv4 = routes.clone();
        let callbacks_ipv6 = callbacks.clone();
        let routes_ipv6 = routes.clone();

        Ok(Self {
            route_monitor_v4: Some(DefaultRouteMonitor::new(
                AddressFamily::Ipv4,
                move |event_type| {
                    Self::default_route_change(&callbacks_ipv4, &routes_ipv4, AF_INET, event_type);
                },
            )?),
            route_monitor_v6: Some(DefaultRouteMonitor::new(
                AddressFamily::Ipv6,
                move |event_type| {
                    Self::default_route_change(&callbacks_ipv6, &routes_ipv6, AF_INET6, event_type);
                },
            )?),
            routes,
            callbacks,
        })
    }

    pub fn add_routes(&self, new_routes: Vec<Route>) -> Result<()> {
        let mut route_manager_routes = self.routes.lock().unwrap();

        let mut event_log = vec![];

        for route in new_routes {
            let registered_route = Self::add_into_routing_table(&route).map_err(|error| {
                if let Err(error) = Self::undo_events(&event_log, &mut route_manager_routes) {
                    error
                } else {
                    error
                }
            })?;

            let new_record = RouteRecord {
                route,
                registered_route,
            };

            event_log.push(EventEntry {
                event_type: RecordEventType::AddRoute,
                record: new_record.clone(),
            });

            let existing_record_idx =
                Self::find_route_record(&mut route_manager_routes, &new_record.registered_route);

            match existing_record_idx {
                None => route_manager_routes.push(new_record),
                Some(idx) => route_manager_routes[idx] = new_record,
            }
        }
        Ok(())
    }

    fn add_into_routing_table(route: &Route) -> Result<RegisteredRoute> {
        let node = Self::resolve_node(ipnetwork_to_address_family(route.network), &route.node)?;

        // SAFETY: MIB_IPFORWARD_ROW2 contains no references or pointers only number primitives and
        // as such it is safe to zero it.
        let mut spec: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };

        // SAFETY: This function must be used to initialize MIB_IPFORWARD_ROW2 structs if it is to
        // be used later by CreateIpForwardEntry2.
        unsafe { InitializeIpForwardEntry(&mut spec) };

        spec.InterfaceLuid = node.iface;
        spec.DestinationPrefix = win_ip_address_prefix_from_ipnetwork_port_zero(route.network);
        spec.NextHop = inet_sockaddr_from_socketaddr(node.gateway);
        spec.Metric = 0;
        spec.Protocol = MIB_IPPROTO_NETMGMT;
        spec.Origin = NlroManual;

        // SAFETY: DestinationPrefix must be initialized to a valid prefix. NextHop must have a
        // valid IP address and family. At least one of InterfaceLuid and InterfaceIndex must be set
        // to the interface.
        let mut status = unsafe { CreateIpForwardEntry2(&spec) };

        // The return code ERROR_OBJECT_ALREADY_EXISTS means there is already an existing route
        // on the same interface, with the same DestinationPrefix and NextHop.
        //
        // However, all the other properties of the route may be different. And the properties may
        // not have the exact same values as when the route was registered, because windows
        // will adjust route properties at time of route insertion as well as later.
        //
        // The simplest thing in this case is to just overwrite the route.
        //

        if ERROR_OBJECT_ALREADY_EXISTS as i32 == status {
            // SAFETY: DestinationPrefix must be initialzed to a valid prefix. NextHop must have
            // a valid IP address and family. At least one of InterfaceLuid and InterfaceIndex must
            // be set to the interface.
            status = unsafe { SetIpForwardEntry2(&spec) };
        }

        if NO_ERROR as i32 != status {
            log::error!("Could not register route in routing table");
            return Err(Error::AddToRouteTable(io::Error::from_raw_os_error(status)));
        }

        Ok(RegisteredRoute {
            network: route.network,
            luid: node.iface,
            next_hop: node.gateway,
        })
    }

    fn resolve_node(family: AddressFamily, optional_node: &NetNode) -> Result<InterfaceAndGateway> {
        // There are four cases:
        //
        // Unspecified node (use interface and gateway of default route).
        // Node is specified by name.
        // Node is specified by name and gateway.
        // Node is specified by gateway.
        //

        match optional_node {
            NetNode::DefaultNode => {
                let default_route = get_best_default_route(family)?;
                match default_route {
                    None => {
                        log::error!("Unable to determine details of default route");
                        return Err(Error::NoDefaultRoute);
                    }
                    Some(default_route) => return Ok(default_route),
                }
            }
            NetNode::RealNode(node) => {
                if let Some(device_name) = &node.get_device() {
                    let device_name = WideCString::from_str(device_name)
                        .expect("Failed to convert UTF-8 string to null terminated UCS string");
                    let luid = match Self::parse_string_encoded_luid(device_name.as_ucstr())? {
                        None => {
                            let mut luid = NET_LUID_LH { Value: 0 };
                            // SAFETY: No specific safety requirement
                            if NO_ERROR as i32
                                != unsafe {
                                    ConvertInterfaceAliasToLuid(device_name.as_ptr(), &mut luid)
                                }
                            {
                                log::error!(
                                    "Unable to derive interface LUID from interface alias: {:?}",
                                    device_name
                                );
                                return Err(Error::DeviceNameNotFound);
                            } else {
                                luid
                            }
                        }
                        Some(luid) => luid,
                    };

                    return Ok(InterfaceAndGateway {
                        iface: luid,
                        gateway: match node.get_address() {
                            Some(ip) => SocketAddr::new(ip, 0),
                            None => match family {
                                AddressFamily::Ipv4 => {
                                    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
                                }
                                AddressFamily::Ipv6 => {
                                    SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
                                }
                            },
                        },
                    });
                }

                // The node is specified only by gateway.
                //

                // Unwrapping is fine because the node must have an address since no device name was
                // found.
                let gateway = node.get_address().map(inet_sockaddr_from_ipaddr).unwrap();
                Ok(InterfaceAndGateway {
                    iface: interface_luid_from_gateway(&gateway)?,
                    gateway: try_socketaddr_from_inet_sockaddr(gateway)
                        .map_err(|_| Error::InvalidSiFamily)?,
                })
            }
        }
    }

    fn find_route_record(records: &mut Vec<RouteRecord>, route: &RegisteredRoute) -> Option<usize> {
        records
            .iter()
            .position(|record| route == &record.registered_route)
    }

    fn undo_events(event_log: &Vec<EventEntry>, records: &mut Vec<RouteRecord>) -> Result<()> {
        // Rewind state by processing events in the reverse order.
        //

        let mut result = Ok(());

        for event in event_log.iter().rev() {
            match event.event_type {
                RecordEventType::AddRoute => {
                    let record_idx = Self::find_route_record(records, &event.record.registered_route)
                        .expect("Internal state inconsistency in route manager, could not find route record");
                    let record = records.get(record_idx)
                        .expect("Internal state inconsistency in route manager, route record index pointing at nothing");

                    if let Err(e) = Self::delete_from_routing_table(&record.registered_route) {
                        result = result.and(Err(e));
                        continue;
                    }
                    records.remove(record_idx);
                }
            }
        }

        result
    }

    fn delete_from_routing_table(route: &RegisteredRoute) -> Result<()> {
        // SAFETY: There are no pointers or references inside of MIB_IPFORWARD_ROW2, only primitive
        // numbers as such it is safe to zero it.
        let mut r: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };

        r.InterfaceLuid = route.luid;
        r.DestinationPrefix = win_ip_address_prefix_from_ipnetwork_port_zero(route.network);
        r.NextHop = inet_sockaddr_from_socketaddr(route.next_hop);

        // SAFETY: DestinationPrefix must be initialzed to a valid prefix. NextHop must have
        // a valid IP address and family. At least one of InterfaceLuid and InterfaceIndex must be
        // set to the interface.
        let status = unsafe { DeleteIpForwardEntry2(&r) };

        match u32::try_from(status) {
            Ok(ERROR_NOT_FOUND) => {
                log::warn!("Attempting to delete route which was not present in routing table, ignoring and proceeding. Route: {}", route);
            }
            Ok(NO_ERROR) => (),
            _ => {
                log::error!(
                    "Failed to delete route in routing table. Route: {}, Status: {}",
                    route,
                    status
                );
                return Err(Error::DeleteFromRouteTable(io::Error::from_raw_os_error(
                    status,
                )));
            }
        }

        Ok(())
    }

    fn restore_into_routing_table(route: &RegisteredRoute) -> Result<()> {
        // SAFETY: There are no pointers or references inside of MIB_IPFORWARD_ROW2, only primitive
        // numbers as such it is safe to zero it.
        let mut spec: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };

        // SAFETY: This function must be used to initialize MIB_IPFORWARD_ROW2 structs if it is to
        // be used later by CreateIpForwardEntry2.
        unsafe { InitializeIpForwardEntry(&mut spec) };

        spec.InterfaceLuid = route.luid;
        spec.DestinationPrefix = win_ip_address_prefix_from_ipnetwork_port_zero(route.network);
        spec.NextHop = inet_sockaddr_from_socketaddr(route.next_hop);
        spec.Metric = 0;
        spec.Protocol = MIB_IPPROTO_NETMGMT;
        spec.Origin = NlroManual;

        // SAFETY: DestinationPrefix must be initialized to a valid prefix. NextHop must have a
        // valid IP address and family. At least one of InterfaceLuid and InterfaceIndex must be set
        // to the interface.
        let status = unsafe { CreateIpForwardEntry2(&spec) };

        if NO_ERROR as i32 != status {
            log::error!(
                "Could not register route in routing table. Route: {}, Status: {}",
                route,
                status
            );
            return Err(Error::AddToRouteTable(io::Error::from_raw_os_error(status)));
        }
        Ok(())
    }

    fn parse_string_encoded_luid(encoded_luid: &WideCStr) -> Result<Option<NET_LUID_LH>> {
        // The `#` is a valid character in adapter names so we use `?` instead.
        // The LUID is thus prefixed with `?` and hex encoded and left-padded with zeroes.
        // E.g. `?deadbeefcafebabe` or `?000dbeefcafebabe`.
        //

        const STRING_ENCODED_LUID_LENGTH: usize = 17;

        if encoded_luid.len() != STRING_ENCODED_LUID_LENGTH
            || Some(Ok('?')) != encoded_luid.chars().next()
        {
            return Ok(None);
        }

        let luid = NET_LUID_LH {
            Value: u64::from_str_radix(
                &encoded_luid.to_string().map_err(|_| {
                    log::error!("Failed to parse string encoded LUID: {:?}", encoded_luid);
                    Error::Conversion
                })?[1..],
                16,
            )
            .map_err(|_| {
                log::error!("Failed to parse string encoded LUID: {:?}", encoded_luid);
                Error::Conversion
            })?,
        };

        return Ok(Some(luid));
    }

    pub fn delete_applied_routes(&mut self) -> Result<()> {
        let mut routes = self.routes.lock().unwrap();
        // Delete all routes owned by us.
        //

        for record in (*routes).iter() {
            if let Err(_) = Self::delete_from_routing_table(&record.registered_route) {
                log::error!(
                    "Failed to delete route while clearing applied routes, {}",
                    record.registered_route
                );
            }
        }

        routes.clear();
        Ok(())
    }

    pub fn register_default_route_changed_callback(&self, callback: Callback) -> CallbackHandle {
        let (nonce, callbacks) = &mut *self.callbacks.lock().unwrap();
        let old_nonce = *nonce;
        callbacks.insert(old_nonce, callback);
        *nonce = nonce.wrapping_add(1);
        CallbackHandle {
            nonce: old_nonce,
            callbacks: self.callbacks.clone(),
        }
    }

    fn default_route_change<'a>(
        callbacks: &Arc<Mutex<(i32, HashMap<i32, Callback>)>>,
        records: &Arc<Mutex<Vec<RouteRecord>>>,
        family: ADDRESS_FAMILY,
        event_type: RouteMonitorEventType<'a>,
    ) {
        // Forward event to all registered listeners.
        //

        {
            let (_, callbacks) = &mut *callbacks.lock().unwrap();
            for callback in callbacks.values() {
                let family =
                    AddressFamily::try_from_af_family(u16::try_from(family).unwrap()).unwrap();
                callback(event_type, family);
            }
        }

        // Examine event to determine if best default route has changed.
        //

        let route = if let RouteMonitorEventType::Updated(route) = event_type {
            route
        } else {
            return;
        };

        // Examine our routes to see if any of them are policy bound to the best default route.
        //

        let mut records = records.lock().unwrap();
        let mut affected_routes: Vec<&mut RouteRecord> = vec![];

        for record in (*records).iter_mut() {
            if matches!(record.route.node, NetNode::DefaultNode)
                && family
                    == u32::from(ipnetwork_to_address_family(record.route.network).to_af_family())
            {
                affected_routes.push(record);
            }
        }

        if affected_routes.is_empty() {
            return;
        }

        // Update all affected routes.
        //

        log::info!("Best default route has changed. Refreshing dependent routes");

        for affected_route in affected_routes {
            // We can't update the existing route because defining characteristics are being
            // changed. So removing and adding again is the only option.
            //

            if let Err(error) = Self::delete_from_routing_table(&affected_route.registered_route) {
                log::error!(
                    "Failed to delete route when refreshing existing routes: {}",
                    error
                );
                continue;
            }

            affected_route.registered_route.luid = route.iface;
            affected_route.registered_route.next_hop = route.gateway;

            if let Err(error) = Self::restore_into_routing_table(&affected_route.registered_route) {
                log::error!(
                    "Failed to add route when refreshing existing routes: {}",
                    error
                );
                continue;
            }
        }
    }
}

impl Drop for RouteManagerInternal {
    fn drop(&mut self) {
        drop(self.route_monitor_v4.take());
        drop(self.route_monitor_v6.take());

        match self.delete_applied_routes() {
            Ok(()) => (),
            Err(e) => {
                log::error!("Failed to correctly drop RouteManagerInternal {}", e)
            }
        }
    }
}

fn interface_luid_from_gateway(gateway: &SOCKADDR_INET) -> Result<NET_LUID_LH> {
    const ADAPTER_FLAGS: GET_ADAPTERS_ADDRESSES_FLAGS = GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST
        | GAA_FLAG_SKIP_DNS_SERVER
        | GAA_FLAG_SKIP_FRIENDLY_NAME
        | GAA_FLAG_INCLUDE_GATEWAYS;

    // SAFETY: The si_family field is always valid to access.
    let family: u32 = u32::from(unsafe { gateway.si_family });
    let adapters = Adapters::new(family, ADAPTER_FLAGS)?;

    // Process adapters to find matching ones.
    //

    let mut matches: Vec<_> = adapters
        // SAFETY: We are not allowed to dereference adapter.Head if it has been aquired in a previous iteration of the iterator
        // we ensure this is upheld by not saving any references to adapter.Head between iterations.
        .iter()
        .filter(|adapter| {
            if !adapter_interface_enabled(adapter, family).unwrap_or(false) {
                return false;
            }
            let gateways = if adapter.FirstGatewayAddress.is_null() {
                vec![]
            } else {
                // SAFETY: adapter.FirstGatewayAddress is not null and all elements in the linked list live
                // in the same buffer and as such have the same lifetime.
                unsafe { isolate_gateway_address(get_first_gateway_address_reference(adapter), family) }
            };

            address_present(gateways, &gateway).unwrap_or(false)
        })
        .collect();

    // Sort matching interfaces ascending by metric.
    //

    let target_v4 = AF_INET == family;

    matches.sort_by(|lhs, rhs| {
        if target_v4 {
            lhs.Ipv4Metric.cmp(&rhs.Ipv4Metric)
        } else {
            lhs.Ipv6Metric.cmp(&rhs.Ipv6Metric)
        }
    });

    // Select the interface with the best (lowest) metric.
    //
    matches
        .get(0)
        .map(|interface| interface.Luid)
        .ok_or_else(|| {
            log::error!("Unable to find network adapter with specified gateway");
            Error::DeviceGatewayNotFound
        })
}

/// SAFETY: adapter.FirstGatewayAddress must be dereferencable and must live as long as adapter
unsafe fn get_first_gateway_address_reference(
    adapter: &IP_ADAPTER_ADDRESSES_LH,
) -> &IP_ADAPTER_GATEWAY_ADDRESS_LH {
    &*adapter.FirstGatewayAddress
}

fn adapter_interface_enabled(
    adapter: &IP_ADAPTER_ADDRESSES_LH,
    family: ADDRESS_FAMILY,
) -> Result<bool> {
    match family {
        // SAFETY: All fields in the Anonymous2 union are at represented by a u32 so dereferencing
        // them is safe
        AF_INET => Ok(0 != unsafe { adapter.Anonymous2.Flags } & IP_ADAPTER_IPV4_ENABLED),
        AF_INET6 => Ok(0 != unsafe { adapter.Anonymous2.Flags } & IP_ADAPTER_IPV6_ENABLED),
        _ => Err(Error::InvalidSiFamily),
    }
}

/// SAFETY: `head` must be a linked list where each `head.Next` is either null or
/// the it and all of its fields has lifetime 'a and are dereferencable.
unsafe fn isolate_gateway_address<'a>(
    head: &'a IP_ADAPTER_GATEWAY_ADDRESS_LH,
    family: ADDRESS_FAMILY,
) -> Vec<&'a SOCKET_ADDRESS> {
    let mut matches = vec![];

    let mut gateway = head;
    loop {
        // SAFETY: The contract states that Address.lpSockaddr is dereferencable if the element is
        // non-null
        if family == u32::from((*gateway.Address.lpSockaddr).sa_family) {
            // SAFETY: The contract states that this field must have lifetime 'a
            matches.push(&gateway.Address);
        }

        if gateway.Next.is_null() {
            break;
        }

        // SAFETY: Gateway.Next is not null here and the contract states it must be dereferencable
        // if non-null
        gateway = &*gateway.Next;
    }

    matches
}

fn address_present(hay: Vec<&'_ SOCKET_ADDRESS>, needle: &'_ SOCKADDR_INET) -> Result<bool> {
    for candidate in hay {
        // SAFETY: Contract states that needle is dereferencable
        if equal_address(needle, candidate)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn equal_address(lhs: &'_ SOCKADDR_INET, rhs: &'_ SOCKET_ADDRESS) -> Result<bool> {
    let rhs = &*rhs;
    // SAFETY: The si_family field is always valid
    if unsafe { lhs.si_family != (*rhs.lpSockaddr).sa_family } {
        return Ok(false);
    }

    match unsafe { lhs.si_family } as u32 {
        AF_INET => {
            let typed_rhs = rhs.lpSockaddr as *mut SOCKADDR_IN;
            // SAFETY: If rhs.lpSockaddr.sa_family is IPv4 then lpSockaddr is a SOCKADDR_IN
            Ok(unsafe { lhs.Ipv4.sin_addr.S_un.S_addr == (*typed_rhs).sin_addr.S_un.S_addr })
        }
        AF_INET6 => {
            let typed_rhs = rhs.lpSockaddr as *mut SOCKADDR_IN6;
            // SAFETY: If rhs.lpSockaddr.sa_family is IPv6 then lpSockaddr is a SOCKADDR_IN6
            Ok(unsafe { lhs.Ipv6.sin6_addr.u.Byte == (*typed_rhs).sin6_addr.u.Byte })
        }
        _ => {
            log::error!("Missing case handler in match");
            Err(Error::InvalidSiFamily)
        }
    }
}

/// Linked list containing `IP_ADAPTER_ADDRESSES_LH` queried from the windows API.
/// Consume by using the iterator produced by `iter_mut()`
struct Adapters {
    // SAFETY: This vector is not allowed to be resized since all of the data inside of it would be
    // dangling
    buffer: Vec<u8>,
}

impl Adapters {
    /// Create a new linked list of adapters from the windows API
    fn new(family: ADDRESS_FAMILY, flags: GET_ADAPTERS_ADDRESSES_FLAGS) -> Result<Self> {
        const MSDN_RECOMMENDED_STARTING_BUFFER_SIZE: usize = 1024 * 15;
        let mut buffer: Vec<u8> = Vec::with_capacity(MSDN_RECOMMENDED_STARTING_BUFFER_SIZE);
        buffer.resize(MSDN_RECOMMENDED_STARTING_BUFFER_SIZE, 0);

        let mut buffer_size = u32::try_from(buffer.len()).unwrap();
        let mut buffer_pointer = buffer.as_mut_ptr();

        // Acquire interfaces.
        //

        loop {
            // SAFETY: buffer_size must point to the correct amount of bytes in the buffer which it
            // does. buffer_pointer must point to the start of a mutable buffer which it
            // does. After this call buffer_size might have changed and as such the
            // buffer must be resized to reflect this if this function is going to be
            // called again.
            let status = unsafe {
                GetAdaptersAddresses(
                    family,
                    flags,
                    std::ptr::null_mut(),
                    buffer_pointer as *mut IP_ADAPTER_ADDRESSES_LH,
                    &mut buffer_size,
                )
            };

            if ERROR_SUCCESS == status {
                // SAFETY: We truncate the buffer to avoid having a bunch of zero:ed objects at the
                // end of it truncate will not change capacity and will therefore
                // never reallocate the vector which means it can not cause the
                // pointers in the buffer to dangle.
                buffer.truncate(usize::try_from(buffer_size).unwrap());
                break;
            }

            if ERROR_NO_DATA == status {
                return Ok(Self { buffer: Vec::new() });
            }

            if ERROR_BUFFER_OVERFLOW != status {
                log::error!("Probe required buffer size for GetAdaptersAddresses");
                return Err(Error::Adapter(io::Error::from_raw_os_error(
                    i32::try_from(status).unwrap(),
                )));
            }

            // The needed length is returned in the buffer_size pointer
            buffer.resize(usize::try_from(buffer_size).unwrap(), 0);
            buffer_pointer = buffer.as_mut_ptr();
        }

        // Verify structure compatibility.
        // The structure has been extended many times.
        //

        // Unwrapping is fine because we previously would return if we got a ERROR_NO_DATA status.
        // As such the buffer is not empty. SAFETY: Casting the buffers first element to an
        // IP_ADAPTER_ADDRESSES_LH is safe as that is the underlying data structure. SAFETY:
        // This union field is always valid to read from
        let system_size = unsafe {
            (*(buffer.get(0).unwrap() as *const u8 as *const IP_ADAPTER_ADDRESSES_LH))
                .Anonymous1
                .Anonymous
                .Length
        };
        let code_size = u32::try_from(std::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>()).unwrap();

        if system_size < code_size {
            log::error!("Expecting IP_ADAPTER_ADDRESSES to have size {code_size} bytes. Found structure with size {system_size} bytes.");
            return Err(Error::Adapter(io::Error::new(io::ErrorKind::Other,
                format!("Expecting IP_ADAPTER_ADDRESSES to have size {code_size} bytes. Found structure with size {system_size} bytes."))));
        }

        // Initialize members.
        //

        Ok(Self { buffer })
    }

    /// Produces a iterator for the linked list in `Adapters` see
    /// [AdaptersIterator](struct.AdaptersIterator.html) SAFETY: See the documentation on
    /// `AdaptersIterator`
    fn iter<'a>(&'a self) -> AdaptersIterator<'a> {
        let cur = if self.buffer.is_empty() {
            std::ptr::null()
        } else {
            &self.buffer[0] as *const u8 as *const IP_ADAPTER_ADDRESSES_LH
        };
        AdaptersIterator {
            _adapters: self,
            cur,
        }
    }
}

/// SAFETY: You are only allowed to dereference `IP_ADAPTER_ADDRESSES_LH.Next` or any following
/// `Next` items in the linked list if they were produced by the latest call to `next()`. Any raw
/// pointers that were aquired before the call to `next()` are not valid to dereference.
struct AdaptersIterator<'a> {
    _adapters: &'a Adapters,
    cur: *const IP_ADAPTER_ADDRESSES_LH,
}

impl<'a> Iterator for AdaptersIterator<'a> {
    type Item = &'a IP_ADAPTER_ADDRESSES_LH;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.is_null() {
            None
        } else {
            let ret = self.cur;
            // SAFETY: self.cur is guaranteed to not be null, we are also holding a &Adapters which
            // guarantees no other reference of self could be held right now which has
            // mutably dereferenced the same address that self.cur is pointing to.
            //
            // It is possible that someone has copied the previous returned items `Next` pointer
            // which points to the same as address as self.cur, however dereferencing
            // that is unsafe and that code is responsible for not dereferencing
            // `Next` on a reference returned by this function after that reference has been
            // dropped.
            self.cur = unsafe { (*self.cur).Next };
            // SAFETY: ret is guaranteed to be non-null and valid since self.adapters owns the
            // memory.
            Some(unsafe { &*ret })
        }
    }
}

/// Convert to a windows defined `IP_ADDRESS_PREFIX` from a `ipnetwork::IpNetwork` but set the port
/// to 0
pub fn win_ip_address_prefix_from_ipnetwork_port_zero(from: IpNetwork) -> IP_ADDRESS_PREFIX {
    // Port should not matter so we set it to 0
    let prefix =
        talpid_windows_net::inet_sockaddr_from_socketaddr(std::net::SocketAddr::new(from.ip(), 0));
    IP_ADDRESS_PREFIX {
        Prefix: prefix,
        PrefixLength: from.prefix(),
    }
}

/// Convert to a windows defined `SOCKADDR_INET` from a `IpAddr` but set the port to 0
pub fn inet_sockaddr_from_ipaddr(from: IpAddr) -> SOCKADDR_INET {
    // Port should not matter so we set it to 0
    talpid_windows_net::inet_sockaddr_from_socketaddr(std::net::SocketAddr::new(from, 0))
}

/// Convert to a `AddressFamily` from a `ipnetwork::IpNetwork`
pub fn ipnetwork_to_address_family(from: IpNetwork) -> AddressFamily {
    if from.is_ipv4() {
        AddressFamily::Ipv4
    } else {
        AddressFamily::Ipv6
    }
}
