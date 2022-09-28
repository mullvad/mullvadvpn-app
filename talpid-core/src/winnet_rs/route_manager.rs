use super::default_route_monitor::{DefaultRouteMonitor, EventType as RouteMonitorEventType};
use super::{Result, WinNetDefaultRoute, get_best_default_route_internal, InterfaceAndGateway, Error};
use crate::windows::AddressFamily;
use windows_sys::Win32::{Foundation::{ERROR_OBJECT_ALREADY_EXISTS, NO_ERROR, ERROR_NOT_FOUND, ERROR_SUCCESS, ERROR_NO_DATA, ERROR_BUFFER_OVERFLOW},
Networking::WinSock::{SOCKADDR_INET, SOCKADDR_IN, SOCKADDR_IN6, SOCKET_ADDRESS, MIB_IPPROTO_NETMGMT, NlroManual, ADDRESS_FAMILY, AF_INET, AF_INET6}, 
    NetworkManagement::IpHelper::
    {IP_ADDRESS_PREFIX, NET_LUID_LH, InitializeIpForwardEntry, MIB_IPFORWARD_ROW2, CreateIpForwardEntry2,
        SetIpForwardEntry2, ConvertInterfaceAliasToLuid, DeleteIpForwardEntry2, GetAdaptersAddresses, GET_ADAPTERS_ADDRESSES_FLAGS,
	IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_GATEWAY_ADDRESS_LH,
GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_DNS_SERVER,
    		 GAA_FLAG_SKIP_FRIENDLY_NAME, GAA_FLAG_INCLUDE_GATEWAYS, IP_ADAPTER_IPV4_ENABLED, IP_ADAPTER_IPV6_ENABLED}};
use widestring::U16CStr;
use std::sync::{Arc, Mutex};

type Network = IP_ADDRESS_PREFIX;
type NodeAddress = SOCKADDR_INET;

struct Adapters {
	buffer: Vec<IP_ADAPTER_ADDRESSES_LH>,
}

impl Adapters {
	fn new(family: ADDRESS_FAMILY, flags: GET_ADAPTERS_ADDRESSES_FLAGS) -> Result<Self> {
		const MSDN_RECOMMENDED_STARTING_BUFFER_SIZE: usize = 1024 * 15;
		let mut buffer = Vec::with_capacity(MSDN_RECOMMENDED_STARTING_BUFFER_SIZE);
		buffer.resize(MSDN_RECOMMENDED_STARTING_BUFFER_SIZE, unsafe { std::mem::zeroed() });

		let mut buffer_size = u32::try_from(buffer.len()).unwrap();
		let mut buffer_pointer = buffer.as_mut_ptr();

		//
		// Acquire interfaces.
		//

		loop {
			let status = unsafe { GetAdaptersAddresses(family, flags, 
				std::ptr::null_mut() as *mut _, buffer_pointer, &mut buffer_size) };

			if ERROR_SUCCESS == status
			{
				// FIXME: If we insert too many objects in the start we will have a bunch of uninitialized zeroed objects
				// at the end of the vector. We should cosider truncating the vector to the right size here.
				break;
			}

			if ERROR_NO_DATA == status
			{
				return Ok(Self { buffer: Vec::new() });
			}

			if ERROR_BUFFER_OVERFLOW != status
			{
				return Err(Error::WindowsApi);
				//THROW_WINDOWS_ERROR(status, "Probe required buffer size for GetAdaptersAddresses");
			}


			// The needed length is returned in the buffer_size pointer
			buffer.resize(usize::try_from(buffer_size).unwrap(), unsafe { std::mem::zeroed() });
			buffer_pointer = buffer.as_mut_ptr();
		}

		//
		// Verify structure compatibility.
		// The structure has been extended many times.
		//

		let system_size = buffer.len();
		let code_size = std::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>();

		if system_size < code_size
		{
			return Err(Error::WindowsApi);
			//std::stringstream ss;

			//ss << "Expecting IP_ADAPTER_ADDRESSES to have size " << codeSize << " bytes. "
			//	<< "Found structure with size " << systemSize << " bytes.";

			//THROW_ERROR(ss.str().c_str());
		}

		//
		// Initialize members.
		//

		Ok(Self { buffer })
	}

	fn iter<'a>(&'a self) -> AdaptersIterator<'a> {
		AdaptersIterator { adapters: self, cur: 0 }
	}
}

struct AdaptersIterator<'a> {
	adapters: &'a Adapters,
	cur: usize,
}

impl<'a> Iterator for AdaptersIterator<'a> {
	type Item = &'a IP_ADAPTER_ADDRESSES_LH;
	fn next(&mut self) -> Option<Self::Item> {
		if self.adapters.buffer.len() >= self.cur {
			None
		} else {
			let ret = self.adapters.buffer.get(self.cur);
			self.cur += 1;
			ret
		}
	}
}

#[derive(Clone)]
struct RegisteredRoute {
    network: Network,
    luid: NET_LUID_LH,
    next_hop: NodeAddress,
}

// FIXME: This might be an invalid implementation
impl PartialEq for RegisteredRoute {
	fn eq(&self, other: &Self) -> bool {
		unsafe { self.luid.Value == other.luid.Value }
	}
}

#[derive(Clone)]
struct Node {
    device_name: Option<widestring::U16CString>,
    gateway: Option<NodeAddress>,
}

#[derive(Clone)]
struct Route {
    network: Network,
    node: Option<Node>,
}

#[derive(Clone)]
struct RouteRecord {
    route: Route,
    registered_route: RegisteredRoute,
}

struct EventEntry {
    record: RouteRecord,
    event_type: EventType,
}

enum EventType {
    AddRoute,
    DeleteRoute,
}

type Callback = Box<dyn Fn(RouteMonitorEventType, AddressFamily, &Option<InterfaceAndGateway>) -> Result<()> + Send>;

pub struct RouteManager {
    route_monitor_v4: DefaultRouteMonitor,
    route_monitor_v6: DefaultRouteMonitor,
	routes: Arc<Mutex<Vec<RouteRecord>>>,
	callbacks: Arc<Mutex<Vec<Callback>>>,
}

impl RouteManager {
    fn new() -> Result<Self> {
		let routes = Arc::new(Mutex::new(
            	// TODO: C++ used a double linked list without explicit initialization
				Vec::new()
			));
		let callbacks = Arc::new(Mutex::new(
				Vec::new()
			));
		let callbacks_ipv4 = callbacks.clone();
		let routes_ipv4 = routes.clone();
		let callbacks_ipv6 = callbacks.clone();
		let routes_ipv6 = routes.clone();
        Ok(Self {
            route_monitor_v4: DefaultRouteMonitor::new(
                AddressFamily::Ipv4,
                move |event_type, route| {
					Self::default_route_change(&callbacks_ipv4, &routes_ipv4, AF_INET, event_type, route);
				},
            )?,
            route_monitor_v6: DefaultRouteMonitor::new(
                AddressFamily::Ipv6,
                move |event_type, route| {
					Self::default_route_change(&callbacks_ipv6, &routes_ipv6, AF_INET6, event_type, route);
				},
            )?,
			routes,
            callbacks,
        })
    }

    fn default_route_changed(
        family: AddressFamily,
        event_type: EventType,
        route: &Option<WinNetDefaultRoute>,
    ) {
    }

    fn add_routes(&mut self, routes: Vec<Route>) -> Result<()>
    {
        // TODO: This should be done inside of a mutex
    	//AutoLockType lock(m_routesLock);
    
    	let mut event_log = vec![];
    
    	for route in routes {
            let registered_route = match Self::add_into_routing_table(&route) {
                Ok(registered_route) => registered_route,
                Err(e) => {
                    match e {
                        Error::RouteManagerError => {
                            // TODO: Look up why this is important to split these?
                            self.undo_events(&event_log);
                            return Err(e);
                        }
                        _ => {
                            // TODO: Look up why this is important to split these?
                            self.undo_events(&event_log);
                            return Err(e);
                        }
                    }
                }
            };

            let new_record = RouteRecord { route, registered_route };
        
			// FIXME: Clone?
            event_log.push(EventEntry{ event_type: EventType::AddRoute, record: new_record.clone() });
        
            // TODO: make sure this makes sense, not clear if it does
            let existing_record_idx = self.find_route_record(&new_record.registered_route);
        
			let mut routes = self.routes.lock().unwrap();
            match existing_record_idx
            {
                None => routes.push(new_record),
                Some(idx) => routes[idx] = new_record,
            }
        }
        Ok(())
    }

    fn add_into_routing_table(route: &Route) -> Result<RegisteredRoute>
    {
        let node = Self::resolve_node(u32::from(unsafe { route.network.Prefix.si_family }),
		&route.node)?;

        //TODO: Make sure this is safe
        let mut spec: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };

        unsafe {InitializeIpForwardEntry(&mut spec)};

        spec.InterfaceLuid = node.iface;
        spec.DestinationPrefix = route.network;
        spec.NextHop = node.gateway;
        spec.Metric = 0;
        spec.Protocol = MIB_IPPROTO_NETMGMT;
        spec.Origin = NlroManual;

        let mut status = unsafe { CreateIpForwardEntry2(&spec) };

        //
        // The return code ERROR_OBJECT_ALREADY_EXISTS means there is already an existing route
        // on the same interface, with the same DestinationPrefix and NextHop.
        //
        // However, all the other properties of the route may be different. And the properties may
        // not have the exact same values as when the route was registered, because windows
        // will adjust route properties at time of route insertion as well as later.
        //
        // The simplest thing in this case is to just overwrite the route.
        //

        if ERROR_OBJECT_ALREADY_EXISTS as i32 == status
        {
            status = unsafe {SetIpForwardEntry2(&spec)};
        }

        if NO_ERROR as i32 != status
        {
            //THROW_WINDOWS_ERROR(status, "Register route in routing table");
            return Err(Error::WindowsApi);
        }

        Ok(RegisteredRoute { network: route.network, luid: node.iface, next_hop: node.gateway })
    }

    fn resolve_node(family: ADDRESS_FAMILY, optional_node: &Option<Node>) -> Result<InterfaceAndGateway>
    {
    	//
    	// There are four cases:
    	//
    	// Unspecified node (use interface and gateway of default route).
    	// Node is specified by name.
    	// Node is specified by name and gateway.
    	// Node is specified by gateway.
    	//

        match optional_node {
            None => {
    		    let default_route = get_best_default_route_internal(AddressFamily::try_from_af_family(u16::try_from(family).unwrap()).unwrap())?;
                match default_route {
                    None => {
    			        //THROW_ERROR_TYPE(error::NoDefaultRoute, "Unable to determine details of default route");
                        return Err(Error::NoDefaultRoute);
                    }
                    Some(default_route) => return Ok(default_route)
                }
            }
            Some(node) => {
                if let Some(device_name) = &node.device_name {
                    // TODO: Make sure this is right
                    let luid = match Self::parse_string_encoded_luid(&device_name)? {
                        None => {
                            let mut luid = NET_LUID_LH { Value: 0 };
                            if NO_ERROR as i32 != unsafe { ConvertInterfaceAliasToLuid(device_name.as_ptr(), &mut luid) } {
                                //let msg = format!("Unable to derive interface LUID from interface alias: {}", device_name);
                                return Err(Error::DeviceNameNotFound);
                            } else {
                                luid
                            }
                        }
                        Some(luid) => luid,
                    };

                    return Ok(InterfaceAndGateway { iface: luid, gateway: node.gateway.unwrap_or_else(|| {
                        // TODO: Is this OK? The family will be set but the other information will not be, trying to
                        // access that information would cause UB
                        NodeAddress { si_family: u16::try_from(family).unwrap() }
    			        //NodeAddress onLink = { 0 };
    			        //onLink.si_family = family;

    			        //return onLink;
                    })});
                }

    	        //
    	        // The node is specified only by gateway.
    	        //

    	        Ok(InterfaceAndGateway{ iface: Self::interface_luid_from_gateway(&node.gateway.as_ref().unwrap())?, gateway: node.gateway.unwrap() })
            }
        }
    }

    fn find_route_record(&self, route: &RegisteredRoute) -> Option<usize>
    {
        self.routes.lock().unwrap().iter().position(|record| route == &record.registered_route)
    }

    fn undo_events(&mut self, event_log: &Vec<EventEntry>) -> Result<()>
    {
    	//
    	// Rewind state by processing events in the reverse order.
    	//

		let mut routes = self.routes.lock().unwrap();
    	for event in event_log.iter().rev()
    	{
            match event.event_type {
                EventType::AddRoute => {
                    match self.find_route_record(&event.record.registered_route) {
                        None => {
                            // Log error
                            //THROW_ERROR("Internal state inconsistency in route manager");
                        }
                        Some(record_idx) => {
                            // TODO: make sure this is right
                            let record = routes.get(record_idx).unwrap();
                            Self::delete_from_routing_table(&record.registered_route)?;
                            routes.remove(record_idx);
                        }
                    }
                }
                EventType::DeleteRoute => {
                    Self::restore_into_routing_table(&event.record.registered_route);
					// FIXME: Clone?
                    routes.push(event.record.clone());
                }
            }
    	}
        Ok(())
    }

    fn delete_from_routing_table(route: &RegisteredRoute) -> Result<()>
    {
    	//MIB_IPFORWARD_ROW2 r = { 0};
        // TODO: Make sure is safe and makes sense
        let mut r: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };

    	r.InterfaceLuid = route.luid;
    	r.DestinationPrefix = route.network;
    	r.NextHop = route.next_hop;

    	let mut status = unsafe { DeleteIpForwardEntry2(&r) };

    	if ERROR_NOT_FOUND as i32 == status
    	{
    		status = NO_ERROR as i32;

    		//let err = format!("Attempting to delete route which was not present in routing table, ignoring and proceeding. Route: {}", route);

            // TODO: log
    		//m_logSink->warning(common::string::ToAnsi(err).c_str());
    	}

    	if NO_ERROR as i32 != status
    	{
    		//THROW_WINDOWS_ERROR(status, "Delete route in routing table");
            return Err(Error::WindowsApi);
    	}
        Ok(())
    }

    fn restore_into_routing_table(route: &RegisteredRoute) -> Result<()>
    {
    	//MIB_IPFORWARD_ROW2 spec;
        // TODO: Make sure this is safe and makes sense
        let mut spec: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };
    
    	unsafe { InitializeIpForwardEntry(&mut spec) };
    
    	spec.InterfaceLuid = route.luid;
    	spec.DestinationPrefix = route.network;
    	spec.NextHop = route.next_hop;
    	spec.Metric = 0;
    	spec.Protocol = MIB_IPPROTO_NETMGMT;
    	spec.Origin = NlroManual;
    
    	let status = unsafe { CreateIpForwardEntry2(&spec) };
    
    	if NO_ERROR as i32 != status
    	{
            return Err(Error::WindowsApi);
    		//THROW_WINDOWS_ERROR(status, "Register route in routing table");
    	}
        Ok(())
    }

    fn parse_string_encoded_luid(encoded_luid: &U16CStr) -> Result<Option<NET_LUID_LH>>
    {
    	//
    	// The `#` is a valid character in adapter names so we use `?` instead.
    	// The LUID is thus prefixed with `?` and hex encoded and left-padded with zeroes.
    	// E.g. `?deadbeefcafebabe` or `?000dbeefcafebabe`.
    	//

    	const STRING_ENCODED_LUID_LENGTH: usize = 17;

        // TODO: Make sure this is OK
    	if encoded_luid.len() != STRING_ENCODED_LUID_LENGTH
    		|| Some(Ok('?')) != encoded_luid.chars().next()
    	{
    		return Ok(None);
    	}
        // TODO: Make sure makes sense
        let luid = NET_LUID_LH {Value: u64::from_str_radix(&encoded_luid.to_string().unwrap()[1..], 16).map_err(|_| {
            Error::Conversion
        })? };

    	//try
    	//{
    	//	std::wstringstream ss;

    	//	ss << std::hex << &encodedLuid[1];
    	//	ss >> luid.Value;
    	//}
    	//catch (...)
    	//{
    	//	const auto msg = std::string("Failed to parse string encoded LUID: ")
    	//		.append(common::string::ToAnsi(encodedLuid));

    	//	THROW_ERROR(msg.c_str());
    	//}

        return Ok(Some(luid));
    }

    fn interface_luid_from_gateway(gateway: &NodeAddress) -> Result<NET_LUID_LH>
    {
    	const adapterFlags: GET_ADAPTERS_ADDRESSES_FLAGS = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER
    		| GAA_FLAG_SKIP_FRIENDLY_NAME | GAA_FLAG_INCLUDE_GATEWAYS;

    	let adapters = Adapters::new(unsafe { gateway.si_family }.into(), adapterFlags)?;

    	//
    	// Process adapters to find matching ones.
    	//

		let mut matches: Vec<_> = adapters.iter().filter(|adapter| {
			match adapter_interface_enabled(adapter, unsafe { gateway.si_family }.into()) {
				Ok(b) => if !b {
					return false;
				}
				Err(_) => return false,
			}

    		let gateways = unsafe { isolate_gateway_address(adapter.FirstGatewayAddress, gateway.si_family.into()) };

			match unsafe { address_present(gateways, &gateway) } {
				Ok(b) => b,
				Err(_) => false,
			}
		}).collect();

    	if matches.is_empty()
    	{
			return Err(Error::RouteManagerError);
    		//THROW_ERROR_TYPE(error::DeviceGatewayNotFound, "Unable to find network adapter with specified gateway");
    	}

    	//
    	// Sort matching interfaces ascending by metric.
    	//

    	let targetV4 = AF_INET == u32::from(unsafe { gateway.si_family } );

		matches.sort_by(|lhs, rhs| {
			if targetV4 {
				lhs.Ipv4Metric.cmp(&rhs.Ipv4Metric)
			} else {
				lhs.Ipv6Metric.cmp(&rhs.Ipv6Metric)
			}
		});

    	//
    	// Select the interface with the best (lowest) metric.
    	//

    	Ok(matches[0].Luid)
    }

    fn delete_applied_routes(&mut self) -> Result<()>
    {
		let mut routes = self.routes.lock().unwrap();
    	//
    	// Delete all routes owned by us.
    	//

    	for record in (*routes).iter()
    	{
    		if let Err(e) = Self::delete_from_routing_table(&record.registered_route) {
                // TODO: Log         
    			//std::wstringstream ss;

    			//ss << L"Failed to delete route while clearing applied routes, Route: "
    			//	<< FormatRegisteredRoute(record.registeredRoute);

    			//m_logSink->error(common::string::ToAnsi(ss.str()).c_str());
    			//m_logSink->error(ex.what());
            }
    	}

    	routes.clear();
		Ok(())
    }
    fn default_route_change(
		callbacks: &Arc<Mutex<Vec<Callback>>>,
		records: &Arc<Mutex<Vec<RouteRecord>>>,
		family: ADDRESS_FAMILY,
		eventType: RouteMonitorEventType,
    	route: &Option<InterfaceAndGateway>
	)
    {
    	//
    	// Forward event to all registered listeners.
    	//

    	//m_defaultRouteCallbacksLock.lock();

		{
			let callbacks = callbacks.lock().unwrap();
			for callback in (*callbacks).iter()
			{
				let family = AddressFamily::try_from_af_family(u16::try_from(family).unwrap()).unwrap();
				match callback(eventType, family, route) {
					Ok(()) => (),
					Err(_) => {
						// TODO: log
						//catch (const std::exception &ex)
						//{
						//	const auto msg = std::string("Failure in default-route-changed callback: ").append(ex.what());
						//	m_logSink->error(msg.c_str());
						//}
						//catch (...)
						//{
						//	m_logSink->error("Unspecified failure in default-route-changed callback");
						//}
					}
				}
			}
		}

    	//m_defaultRouteCallbacksLock.unlock();

    	//
    	// Examine event to determine if best default route has changed.
    	//

    	if RouteMonitorEventType::Updated != eventType
    	{
    		return;
    	}

    	//
    	// Examine our routes to see if any of them are policy bound to the best default route.
    	//

		let mut records = records.lock().unwrap();
    	//AutoLockType routesLock(m_routesLock);

    	//using RecordIterator = std::list<RouteRecord>::iterator;

    	//std::list<RecordIterator> affectedRoutes;
		let mut affected_routes: Vec<&mut RouteRecord> = vec![];

    	//for (RecordIterator it = m_routes.begin(); it != m_routes.end(); ++it)
    	for record in (*records).iter_mut()
    	{
    		if record.route.node.is_none()
    			&& family == u32::from(unsafe { record.route.network.Prefix.si_family })
    		{
    			affected_routes.push(record);
    		}
    	}

    	if affected_routes.is_empty()
    	{
    		return;
    	}

    	//
    	// Update all affected routes.
    	//

		//TODO: Log
    	//m_logSink->info("Best default route has changed. Refreshing dependent routes");

    	for affected_route in affected_routes
    	{
    		//
    		// We can't update the existing route because defining characteristics are being changed.
    		// So removing and adding again is the only option.
    		//

    		match Self::delete_from_routing_table(&affected_route.registered_route) {
				Ok(()) => (),
				Err(e) => {
    				//catch (const std::exception &ex)
    				//{
    				//	const auto msg = std::string("Failed to delete route when refreshing " \
    				//		"existing routes: ").append(ex.what());

    				//	m_logSink->error(msg.c_str());

    				//	continue;
    				//}
					//TODO: log
					continue;
				}

			}
			// FIXME: What if it is None here?
    		affected_route.registered_route.luid = route.as_ref().unwrap().iface;
    		affected_route.registered_route.next_hop = route.as_ref().unwrap().gateway;

    		match Self::restore_into_routing_table(&affected_route.registered_route) {
				Ok(()) => (),
				Err(e) => {
    				//catch (const std::exception &ex)
    				//{
    				//	const auto msg = std::string("Failed to add route when refreshing " \
    				//		"existing routes: ").append(ex.what());

    				//	m_logSink->error(msg.c_str());

    				//	continue;
    				//}
					// TODO: Log
					continue;
				}
			}
    	}
    }
}


impl std::ops::Drop for RouteManager {
    fn drop(&mut self) {
        self.delete_applied_routes();
    }
}

fn adapter_interface_enabled(adapter: &IP_ADAPTER_ADDRESSES_LH, family: ADDRESS_FAMILY) -> Result<bool>
{
	match family
	{
		AF_INET =>
		{
			//Ok(0 != adapter.Ipv4Enabled)
			Ok(0 != unsafe {adapter.Anonymous2.Flags } & IP_ADAPTER_IPV4_ENABLED)
		}
		AF_INET6 =>
		{
			//Ok(0 != adapter.Ipv6Enabled)
			Ok(0 != unsafe { adapter.Anonymous2.Flags } & IP_ADAPTER_IPV6_ENABLED)
		}
		_ =>
		{
			//THROW_ERROR("Missing case handler in switch clause");
			Err(Error::InvalidSiFamily)
		}
	}
}

/// SAFETY: All elements in the linked list pointed to by `head` must outlive the raw pointers returned by this function
/// Furthermore No element in `head` may be mutated until all raw pointers returned by this function have been dropped.
unsafe fn isolate_gateway_address
(
	head: *mut IP_ADAPTER_GATEWAY_ADDRESS_LH,
	family: ADDRESS_FAMILY,
) -> Vec<*const SOCKET_ADDRESS>
{
	let mut matches = vec![];

	let mut gateway_ptr = head;
	// FIXME: This makes us miss the first gateway
	//for (auto gateway = head; nullptr != gateway; gateway = gateway->Next)
	loop
	{
		if gateway_ptr.is_null() {
			break
		}

		let gateway = unsafe { *gateway_ptr };
		if family == u32::from(unsafe { (*gateway.Address.lpSockaddr).sa_family })
		{
			// TODO: makes sense?
			matches.push(&gateway.Address as *const _);
		}

		gateway_ptr = gateway.Next;
	}

	return matches;
}

// SAFETY: All raw pointers in `hay` must be dereferencable
unsafe fn address_present(hay: Vec<*const SOCKET_ADDRESS>, needle: &SOCKADDR_INET) -> Result<bool>
{
	for candidate in hay
	{
		if equal_address(needle, candidate)?
		{
			return Ok(true);
		}
	}

	return Ok(false);
}

// SAFETY: rhs must be dereferencable
unsafe fn equal_address(lhs: &SOCKADDR_INET, rhs: *const SOCKET_ADDRESS) -> Result<bool>
{
	let rhs = &*rhs;
	if unsafe { lhs.si_family != (*rhs.lpSockaddr).sa_family }
	{
		return Ok(false);
	}

	match lhs.si_family as u32
	{
		AF_INET =>
		{
			//let typedRhs = reinterpret_cast<const SOCKADDR_IN *>(rhs.lpSockaddr);
			// FIXME: Make this not transmute, there are likely better ways
			let typedRhs = rhs.lpSockaddr as *mut SOCKADDR_IN;
			Ok(unsafe { lhs.Ipv4.sin_addr.S_un.S_addr == (*typedRhs).sin_addr.S_un.S_addr })
		}
		AF_INET6 =>
		{
			//let typedRhs = reinterpret_cast<const SOCKADDR_IN6 *>(rhs->lpSockaddr);
			// FIXME: Make this not transmute, there are likely better ways
			let typedRhs = rhs.lpSockaddr as *mut SOCKADDR_IN6;
			//return 0 == memcmp(lhs->Ipv6.sin6_addr.u.Byte, typedRhs->sin6_addr.u.Byte, 16);
			Ok(unsafe { lhs.Ipv6.sin6_addr.u.Byte == (*typedRhs).sin6_addr.u.Byte })
		}
		_ =>
		{
			//THROW_ERROR("Missing case handler in switch clause");
			Err(Error::InvalidSiFamily)
		}
	}
}

//RouteManager::~RouteManager()
//{
//	//
//	// Stop callbacks that are triggered by events in Windows from coming in.
//	//
//
//	m_routeMonitorV4.reset();
//	m_routeMonitorV6.reset();
//
//	deleteAppliedRoutes();
//}
//void RouteManager::deleteAppliedRoutes()
//{
//	//
//	// Delete all routes owned by us.
//	//
//
//	for (const auto &record : m_routes)
//	{
//		try
//		{
//			deleteFromRoutingTable(record.registeredRoute);
//		}
//		catch (const std::exception & ex)
//		{
//			std::wstringstream ss;
//
//			ss << L"Failed to delete route while clearing applied routes, Route: "
//				<< FormatRegisteredRoute(record.registeredRoute);
//
//			m_logSink->error(common::string::ToAnsi(ss.str()).c_str());
//			m_logSink->error(ex.what());
//		}
//	}
//
//	m_routes.clear();
//}

//using AutoLockType = std::scoped_lock<std::mutex>;
//using AutoRecursiveLockType = std::scoped_lock<std::recursive_mutex>;
//using namespace std::placeholders;
//
//namespace winnet::routing
//{
//
//namespace
//{
//
//using Adapters = common::network::Adapters;
//
//NET_LUID InterfaceLuidFromGateway(const NodeAddress &gateway)
//{
//	const DWORD adapterFlags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER
//		| GAA_FLAG_SKIP_FRIENDLY_NAME | GAA_FLAG_INCLUDE_GATEWAYS;
//
//	Adapters adapters(gateway.si_family, adapterFlags);
//
//	//
//	// Process adapters to find matching ones.
//	//
//
//	std::vector<const IP_ADAPTER_ADDRESSES *> matches;
//
//	for (auto adapter = adapters.next(); nullptr != adapter; adapter = adapters.next())
//	{
//		if (false == AdapterInterfaceEnabled(adapter, gateway.si_family))
//		{
//			continue;
//		}
//
//		auto gateways = IsolateGatewayAddresses(adapter->FirstGatewayAddress, gateway.si_family);
//
//		if (AddressPresent(gateways, &gateway))
//		{
//			matches.emplace_back(adapter);
//		}
//	}
//
//	if (matches.empty())
//	{
//		THROW_ERROR_TYPE(error::DeviceGatewayNotFound, "Unable to find network adapter with specified gateway");
//	}
//
//	//
//	// Sort matching interfaces ascending by metric.
//	//
//
//	const bool targetV4 = (AF_INET == gateway.si_family);
//
//	std::sort(matches.begin(), matches.end(), [&targetV4](const IP_ADAPTER_ADDRESSES *lhs, const IP_ADAPTER_ADDRESSES *rhs)
//	{
//		if (targetV4)
//		{
//			return lhs->Ipv4Metric < rhs->Ipv4Metric;
//		}
//
//		return lhs->Ipv6Metric < rhs->Ipv6Metric;
//	});
//
//	//
//	// Select the interface with the best (lowest) metric.
//	//
//
//	return matches[0]->Luid;
//}
//
//bool ParseStringEncodedLuid(const std::wstring &encodedLuid, NET_LUID &luid)
//{
//	//
//	// The `#` is a valid character in adapter names so we use `?` instead.
//	// The LUID is thus prefixed with `?` and hex encoded and left-padded with zeroes.
//	// E.g. `?deadbeefcafebabe` or `?000dbeefcafebabe`.
//	//
//
//	static const size_t StringEncodedLuidLength = 17;
//
//	if (encodedLuid.size() != StringEncodedLuidLength
//		|| L'?' != encodedLuid[0])
//	{
//		return false;
//	}
//
//	try
//	{
//		std::wstringstream ss;
//
//		ss << std::hex << &encodedLuid[1];
//		ss >> luid.Value;
//	}
//	catch (...)
//	{
//		const auto msg = std::string("Failed to parse string encoded LUID: ")
//			.append(common::string::ToAnsi(encodedLuid));
//
//		THROW_ERROR(msg.c_str());
//	}
//
//	return true;
//}
//
//InterfaceAndGateway ResolveNode(ADDRESS_FAMILY family, const std::optional<Node> &optionalNode)
//{
//	//
//	// There are four cases:
//	//
//	// Unspecified node (use interface and gateway of default route).
//	// Node is specified by name.
//	// Node is specified by name and gateway.
//	// Node is specified by gateway.
//	//
//
//	if (false == optionalNode.has_value())
//	{
//		const auto default_route = GetBestDefaultRoute(family);
//		if (!default_route.has_value())
//		{
//			THROW_ERROR_TYPE(error::NoDefaultRoute, "Unable to determine details of default route");
//		}
//		return default_route.value();
//	}
//
//	const auto &node = optionalNode.value();
//
//	if (node.deviceName().has_value())
//	{
//		const auto &deviceName = node.deviceName().value();
//		NET_LUID luid;
//
//		if (false == ParseStringEncodedLuid(deviceName, luid)
//			&& 0 != ConvertInterfaceAliasToLuid(deviceName.c_str(), &luid))
//		{
//			const auto msg = std::string("Unable to derive interface LUID from interface alias: ")
//				.append(common::string::ToAnsi(deviceName));
//			THROW_ERROR_TYPE(error::DeviceNameNotFound, msg.c_str());
//		}
//
//		auto onLinkProvider = [&family]()
//		{
//			NodeAddress onLink = { 0 };
//			onLink.si_family = family;
//
//			return onLink;
//		};
//
//		return InterfaceAndGateway{ luid, node.gateway().value_or(onLinkProvider()) };
//	}
//
//	//
//	// The node is specified only by gateway.
//	//
//
//	return InterfaceAndGateway{ InterfaceLuidFromGateway(node.gateway().value()), node.gateway().value() };
//}
//
//std::wstring FormatNetwork(const Network &network)
//{
//	using namespace common::string;
//
//	switch (network.Prefix.si_family)
//	{
//		case AF_INET:
//		{
//			return FormatIpv4<AddressOrder::NetworkByteOrder>(network.Prefix.Ipv4.sin_addr.s_addr, network.PrefixLength);
//		}
//		case AF_INET6:
//		{
//			return FormatIpv6(network.Prefix.Ipv6.sin6_addr.u.Byte, network.PrefixLength);
//		}
//		default:
//		{
//			return L"Failed to format network details";
//		}
//	}
//}
//
//} // anonymous namespace
//
//RouteManager::~RouteManager()
//{
//	//
//	// Stop callbacks that are triggered by events in Windows from coming in.
//	//
//
//	m_routeMonitorV4.reset();
//	m_routeMonitorV6.reset();
//
//	deleteAppliedRoutes();
//}
//
//void RouteManager::addRoutes(const std::vector<Route> &routes)
//{
//	AutoLockType lock(m_routesLock);
//
//	std::vector<EventEntry> eventLog;
//
//	for (const auto &route : routes)
//	{
//		try
//		{
//			RouteRecord newRecord{ route, addIntoRoutingTable(route) };
//
//			eventLog.emplace_back(EventEntry{ EventType::ADD_ROUTE, newRecord });
//
//			auto existingRecord = findRouteRecord(newRecord.registeredRoute);
//
//			if (m_routes.end() == existingRecord)
//			{
//				m_routes.emplace_back(std::move(newRecord));
//			}
//			else
//			{
//				*existingRecord = std::move(newRecord);
//			}
//		}
//		catch (const error::RouteManagerError&)
//		{
//			undoEvents(eventLog);
//			throw;
//		}
//		catch (...)
//		{
//			undoEvents(eventLog);
//			THROW_ERROR("Unexpected error during batch insertion of routes");
//		}
//	}
//}
//
//void RouteManager::deleteRoutes(const std::vector<Route> &routes)
//{
//	AutoLockType lock(m_routesLock);
//
//	std::vector<EventEntry> eventLog;
//
//	for (const auto &route : routes)
//	{
//		try
//		{
//			const auto record = findRouteRecordFromSpec(route);
//
//			if (m_routes.end() == record)
//			{
//				const auto err = std::wstring(L"Request to delete unknown route: ")
//					.append(FormatNetwork(route.network()));
//
//				m_logSink->warning(common::string::ToAnsi(err).c_str());
//
//				continue;
//			}
//
//			deleteFromRoutingTable(record->registeredRoute);
//
//			eventLog.emplace_back(EventEntry{ EventType::DELETE_ROUTE, *record });
//			m_routes.erase(record);
//		}
//		catch (...)
//		{
//			undoEvents(eventLog);
//
//			THROW_ERROR("Failed during batch removal of routes");
//		}
//	}
//}
//
//
//RouteManager::CallbackHandle RouteManager::registerDefaultRouteChangedCallback(DefaultRouteChangedCallback callback)
//{
//	AutoRecursiveLockType lock(m_defaultRouteCallbacksLock);
//
//	m_defaultRouteCallbacks.emplace_back(callback);
//
//	// Return raw address of record in list.
//	return &m_defaultRouteCallbacks.back();
//}
//
//void RouteManager::unregisterDefaultRouteChangedCallback(CallbackHandle handle)
//{
//	AutoRecursiveLockType lock(m_defaultRouteCallbacksLock);
//
//	for (auto it = m_defaultRouteCallbacks.begin(); it != m_defaultRouteCallbacks.end(); ++it)
//	{
//		// Match on raw address of record.
//		if (&*it == handle)
//		{
//			m_defaultRouteCallbacks.erase(it);
//			return;
//		}
//	}
//}
//
//std::list<RouteManager::RouteRecord>::iterator RouteManager::findRouteRecord(const RegisteredRoute &route)
//{
//	return std::find_if(m_routes.begin(), m_routes.end(), [&route](const auto &record)
//	{
//		return route == record.registeredRoute;
//	});
//}
//
//std::list<RouteManager::RouteRecord>::iterator RouteManager::findRouteRecordFromSpec(const Route &route)
//{
//	return std::find_if(m_routes.begin(), m_routes.end(), [&route](const auto &record)
//	{
//		return route == record.route;
//	});
//}
//
//RouteManager::RegisteredRoute RouteManager::addIntoRoutingTable(const Route &route)
//{
//	const auto node = ResolveNode(route.network().Prefix.si_family, route.node());
//
//	MIB_IPFORWARD_ROW2 spec;
//
//	InitializeIpForwardEntry(&spec);
//
//	spec.InterfaceLuid = node.iface;
//	spec.DestinationPrefix = route.network();
//	spec.NextHop = node.gateway;
//	spec.Metric = 0;
//	spec.Protocol = MIB_IPPROTO_NETMGMT;
//	spec.Origin = NlroManual;
//
//	auto status = CreateIpForwardEntry2(&spec);
//
//	//
//	// The return code ERROR_OBJECT_ALREADY_EXISTS means there is already an existing route
//	// on the same interface, with the same DestinationPrefix and NextHop.
//	//
//	// However, all the other properties of the route may be different. And the properties may
//	// not have the exact same values as when the route was registered, because windows
//	// will adjust route properties at time of route insertion as well as later.
//	//
//	// The simplest thing in this case is to just overwrite the route.
//	//
//
//	if (status == ERROR_OBJECT_ALREADY_EXISTS)
//	{
//		status = SetIpForwardEntry2(&spec);
//	}
//
//	if (NO_ERROR != status)
//	{
//		THROW_WINDOWS_ERROR(status, "Register route in routing table");
//	}
//
//	return RegisteredRoute { route.network(), node.iface, node.gateway };
//}
//
//void RouteManager::restoreIntoRoutingTable(const RegisteredRoute &route)
//{
//	MIB_IPFORWARD_ROW2 spec;
//
//	InitializeIpForwardEntry(&spec);
//
//	spec.InterfaceLuid = route.luid;
//	spec.DestinationPrefix = route.network;
//	spec.NextHop = route.nextHop;
//	spec.Metric = 0;
//	spec.Protocol = MIB_IPPROTO_NETMGMT;
//	spec.Origin = NlroManual;
//
//	const auto status = CreateIpForwardEntry2(&spec);
//
//	if (NO_ERROR != status)
//	{
//		THROW_WINDOWS_ERROR(status, "Register route in routing table");
//	}
//}
//
//void RouteManager::deleteFromRoutingTable(const RegisteredRoute &route)
//{
//	MIB_IPFORWARD_ROW2 r = { 0};
//
//	r.InterfaceLuid = route.luid;
//	r.DestinationPrefix = route.network;
//	r.NextHop = route.nextHop;
//
//	auto status = DeleteIpForwardEntry2(&r);
//
//	if (ERROR_NOT_FOUND == status)
//	{
//		status = NO_ERROR;
//
//		const auto err = std::wstring(L"Attempting to delete route which was not present in routing table, " \
//			"ignoring and proceeding. Route: ").append(FormatRegisteredRoute(route));
//
//		m_logSink->warning(common::string::ToAnsi(err).c_str());
//	}
//
//	if (NO_ERROR != status)
//	{
//		THROW_WINDOWS_ERROR(status, "Delete route in routing table");
//	}
//}
//
//void RouteManager::undoEvents(const std::vector<EventEntry> &eventLog)
//{
//	//
//	// Rewind state by processing events in the reverse order.
//	//
//
//	for (auto it = eventLog.rbegin(); it != eventLog.rend(); ++it)
//	{
//		try
//		{
//			switch (it->type)
//			{
//				case EventType::ADD_ROUTE:
//				{
//					const auto record = findRouteRecord(it->record.registeredRoute);
//
//					if (m_routes.end() == record)
//					{
//						THROW_ERROR("Internal state inconsistency in route manager");
//					}
//
//					deleteFromRoutingTable(record->registeredRoute);
//					m_routes.erase(record);
//
//					break;
//				}
//				case EventType::DELETE_ROUTE:
//				{
//					restoreIntoRoutingTable(it->record.registeredRoute);
//					m_routes.emplace_back(it->record);
//
//					break;
//				}
//				default:
//				{
//					THROW_ERROR("Missing case handler in switch clause");
//				}
//			}
//		}
//		catch (const std::exception &ex)
//		{
//			const auto err = std::string("Attempting to rollback state: ").append(ex.what());
//			m_logSink->error(err.c_str());
//		}
//	}
//}
//
//// static
//std::wstring RouteManager::FormatRegisteredRoute(const RegisteredRoute &route)
//{
//	using namespace common::string;
//
//	std::wstringstream ss;
//
//	if (AF_INET == route.network.Prefix.si_family)
//	{
//		std::wstring gateway(L"\"On-link\"");
//
//		if (0 != route.nextHop.Ipv4.sin_addr.s_addr)
//		{
//			gateway = FormatIpv4<AddressOrder::NetworkByteOrder>(route.nextHop.Ipv4.sin_addr.s_addr);
//		}
//
//		ss << FormatIpv4<AddressOrder::NetworkByteOrder>(route.network.Prefix.Ipv4.sin_addr.s_addr, route.network.PrefixLength)
//			<< L" with gateway " << gateway
//			<< L" on interface with LUID 0x" << std::hex << route.luid.Value;
//	}
//	else if (AF_INET6 == route.network.Prefix.si_family)
//	{
//		std::wstring gateway(L"\"On-link\"");
//
//		const uint8_t *begin = &route.nextHop.Ipv6.sin6_addr.u.Byte[0];
//		const uint8_t *end = begin + 16;
//
//		if (0 != std::accumulate(begin, end, 0))
//		{
//			gateway = FormatIpv6(route.nextHop.Ipv6.sin6_addr.u.Byte);
//		}
//
//		ss << FormatIpv6(route.network.Prefix.Ipv6.sin6_addr.u.Byte, route.network.PrefixLength)
//			<< L" with gateway " << gateway
//			<< L" on interface with LUID 0x" << std::hex << route.luid.Value;
//	}
//	else
//	{
//		ss << L"Failed to format route details";
//	}
//
//	return ss.str();
//}
//
//void RouteManager::defaultRouteChanged(ADDRESS_FAMILY family, DefaultRouteMonitor::EventType eventType,
//	const std::optional<InterfaceAndGateway> &route)
//{
//	//
//	// Forward event to all registered listeners.
//	//
//
//	m_defaultRouteCallbacksLock.lock();
//
//	for (const auto &callback : m_defaultRouteCallbacks)
//	{
//		try
//		{
//			callback(eventType, family, route);
//		}
//		catch (const std::exception &ex)
//		{
//			const auto msg = std::string("Failure in default-route-changed callback: ").append(ex.what());
//			m_logSink->error(msg.c_str());
//		}
//		catch (...)
//		{
//			m_logSink->error("Unspecified failure in default-route-changed callback");
//		}
//	}
//
//	m_defaultRouteCallbacksLock.unlock();
//
//	//
//	// Examine event to determine if best default route has changed.
//	//
//
//	if (DefaultRouteMonitor::EventType::Updated != eventType)
//	{
//		return;
//	}
//
//	//
//	// Examine our routes to see if any of them are policy bound to the best default route.
//	//
//
//	AutoLockType routesLock(m_routesLock);
//
//	using RecordIterator = std::list<RouteRecord>::iterator;
//
//	std::list<RecordIterator> affectedRoutes;
//
//	for (RecordIterator it = m_routes.begin(); it != m_routes.end(); ++it)
//	{
//		if (false == it->route.node().has_value()
//			&& family == it->route.network().Prefix.si_family)
//		{
//			affectedRoutes.emplace_back(it);
//		}
//	}
//
//	if (affectedRoutes.empty())
//	{
//		return;
//	}
//
//	//
//	// Update all affected routes.
//	//
//
//	m_logSink->info("Best default route has changed. Refreshing dependent routes");
//
//	for (auto &it : affectedRoutes)
//	{
//		//
//		// We can't update the existing route because defining characteristics are being changed.
//		// So removing and adding again is the only option.
//		//
//
//		try
//		{
//			deleteFromRoutingTable(it->registeredRoute);
//		}
//		catch (const std::exception &ex)
//		{
//			const auto msg = std::string("Failed to delete route when refreshing " \
//				"existing routes: ").append(ex.what());
//
//			m_logSink->error(msg.c_str());
//
//			continue;
//		}
//
//		it->registeredRoute.luid = route.value().iface;
//		it->registeredRoute.nextHop = route.value().gateway;
//
//		try
//		{
//			restoreIntoRoutingTable(it->registeredRoute);
//		}
//		catch (const std::exception &ex)
//		{
//			const auto msg = std::string("Failed to add route when refreshing " \
//				"existing routes: ").append(ex.what());
//
//			m_logSink->error(msg.c_str());
//
//			continue;
//		}
//	}
//}
//
//}
//