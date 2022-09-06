use windows::Win32::NetworkManagment::IpHelper::{self, GetIpForwardTable2, MIB_IPFORWARD_TABLE2, MIB_IPFORWARD_ROW2, MIB_IF_ROW2, FreeMibTable, GetIpInterfaceEntry};

// Interface description substrings found for virtual adapters.
const TUNNEL_INTERFACE_DESCS: Vec<&[u8]> = [
	b"WireGuard",
	b"Wintun",
	b"Tunnel"
];

// FIXME: We should have a better error
type Result<T> = std::result::Result<T, ()>;

pub enum WinNetIp {
    IPV4([u8; 4]),
    IPV6([u8; 16])
}

pub struct WinNetDefaultRoute {
    pub interface_luid: u64,
    pub gateway: WinNetIp,
}

pub enum WinNetAddrFamily {
    IPV4,
    IPV6,
}


fn ip_from_native(from: &SOCKADDR_INET) -> Result<WinNetIp> {
    if from.si_family == AF_INET.0 {
        Ok(WinNetIp::IPV4(from.Ipv4.sin_addr.S_un.S_addr))
    } else if from.si_family == AF_INET6.0 {
        Ok(WinNetIp::IPV6(from.Ipv6.sin6_addr.u.Byte.clone()))
    } else {
        // FIXME: Should this be a panic instead?
        //panic!("Invalid network address family");
        Err(())
    }
}

pub fn get_best_default_route(family: WinNetAddrFamily) -> Result<WinNetDefaultRoute> {
    let mut default_route = WinNetDefaultRoute::default();
    let iface_and_gateway = get_best_default_internal(family)?.ok_or_else(|| ())?;
    default_route.interface_luid = iface_and_gateway.iface.Value;
    default_route.gateway = ip_from_native(iface_and_gateway.gateway)?;
    Ok(default_route)
}

struct MibIpforwardTable2(MIB_IPFORWARD_TABLE2);

impl MibIpforwardTable2 {
    fn new() -> Result<Self> {
        let table: MIB_IPFORWARD_TABLE2 = MIB_IPFORWARD_TABLE2::default();
        // TODO: Proper error handling
        unsafe { GetIpForwardTable2(family, &mut table as *mut *mut MIB_IPFORWARD_TABLE2).map_err(|_| ())? };
        Ok(Self(table))
    }

    fn get_table_entry(&self, i: usize) -> &MIB_IPFORWARD_ROW2 {
        assert!(i < self.0.NumEntries);
        assert!(i * size_of::<MIB_IPFORWARD_ROW2>() < isize::MAX);
        let ptr = self.0.Table[0].as_ptr();
        let row: &MIB_IPFORWARD_ROW2 = unsafe { ptr.offset(i).as_ref() }.unwrap();
        row
    }
}

impl Deref<MIB_IPFORWARD_TABLE2> for MibIpforwardTable2 {
    fn deref(&self) -> &MIB_IPFORWARD_TABLE2 {
        &self.0
    }
}

impl Drop for MibIpforwardTable2 {
    fn drop(&mut self) {
        unsafe { FreeMibTable(&self.0 as *const _) }
    }
}

struct InterfaceAndGateway {
    iface: NET_LUID_LH,
    gateway: SOCKADDR_INET,
}

fn get_best_default_internal(family: WinNetAddrFamily) -> Result<Option<InterfaceAndGateway>> {
    let table = MibIpforwardTable2::new()?;
    let mut candidates = Vec::with_capacity(*table.NumEntries);

	//
	// Enumerate routes looking for: route 0/0
	// The WireGuard interface route has no gateway.
	//

    for i in 0..*table.NumEntries {
		let candidate = table.get_table_entry(i);

		if (0 == candidate.DestinationPrefix.PrefixLength
			&& route_has_gateway(candidate)
			&& is_route_on_physical_interface(candidate))
		{
			candidates.push(candidate);
		}
	}

	let mut annotated = annotated_routes(&candidates);

	if annotated.empty() {
		return Ok(None);
	}

	//
	// Sort on (active, effectiveMetric) ascending by metric.
	//

    annotated.sort_by(|lhs, rhs| {
        if lhs.active == rhs.active {
            return lhs.effective_metric < rhs.effective_metric;
        }
        return lhs.active && false == rhs.active;
    });

	//
	// Ensure the top rated route is active.
	//

	if (false == annotated[0].active) {
		return Ok(None);
	}

	Ok(Some(InterfaceAndGateway { iface: annotated[0].route.InterfaceLuid, gateway: annotated[0].route.NextHop }))
}


fn route_has_gateway(route: &MIB_IPFORWARD_ROW2) -> bool {
	if route.NextHop.si_family == IpHelper::AF_INET.0 {
        return 0 != route.NextHop.Ipv4.sin_addr.s_addr;
    } else if route.NextHop.si_family == IpHelper::AF_INET.0 {
        return 0 != &route.NextHop.Ipv6.sin6_addr.u.Byte.iter().sum();
    } else {
        return false;
    }
}


fn is_route_on_physical_interface(route: &MIB_IPFORWARD_ROW2) -> Result<bool> {
    // FIXME: This is straying from the original c++ implementation, look up the documentation and
    // make sure that this is right.
    if route.InterfaceLuid.Info._bitfield & IpHelper::IF_TYPE_SOFTWARE_LOOPBACK == IpHelper::IF_TYPE_SOFTWARE_LOOPBACK ||
        route.InterfaceLuid.Info._bitfield & IpHelper::IF_TYPE_TUNNEL == IpHelper::IF_TYPE_TUNNEL {
        return Ok(false);
    }
    
    // OpenVPN uses interface type IF_TYPE_PROP_VIRTUAL,
    // but tethering etc. may rely on virtual adapters too,
    // so we have to filter out the TAP adapter specifically.
    
    let mut row = MIB_IF_ROW2::default();
    row.InterfaceLuid = route.InterfaceLuid;
    
    unsafe { IpHelper::GetIfEntry2(&mut row as *mut MIB_IF_ROW2).map_err(|_| ())? };
    
    for tunnel_interface_desc in TUNNEL_INTERFACE_DESCS {
        // FIXME: Is this right? Does contain actually work here?
    	if row.Description.contains(tunnel_interface_desc) {
    		return Ok(false);
    	}
    }
    
    return Ok(true);
}


struct AnnotatedRoute<'a> {
	route: &'a MIB_IPFORWARD_ROW2,
	active: bool,
	effective_metric: u32
}

fn annotated_routes<'a>(routes: &'_ Vec<&'a MIB_IPFORWARD_ROW2>) -> Vec<AnnotatedRoute<'a>> {
    routes.iter().filter_map(|route| {
        // GetAdapterInterface
        let mut iface = MIB_IPINTERFACE_ROW::default();

        iface.Family = route.DestinationPrefix.Prefix.si_family;
        iface.InterfaceLuid = route.InterfaceLuid;

        match unsafe { GetIpInterfaceEntry(&mut iface as *mut MIB_IPINTERFACE_ROW) } {
            Ok(()) => {
                Some(AnnotatedRoute {
                    route,
                    active: iface.Connected.0 != 0,
                    effective_metric: route.Metric + iface.Metric,
                })
            },
            Err(e) => {
                None
            }
        }
    }).collect()
}
