use windows::Win32::NetworkManagment::IpHelper::{self, GetIpForwardTable2, MIB_IPFORWARD_TABLE2, MIB_IPFORWARD_ROW2, MIB_IF_ROW2};

// Interface description substrings found for virtual adapters.
const TUNNEL_INTERFACE_DESCS: Vec<&[u8]> = [
	b"WireGuard",
	b"Wintun",
	b"Tunnel"
];

// FIXME: We should have a better error
type Result<T> = std::result::Result<T, ()>;

pub struct WinNetIp {
    pub addr_family: WinNetAddrFamily,
    pub ip_bytes: [u8; 16],
}

pub struct WinNetDefaultRoute {
    pub interface_luid: u64,
    pub gateway: WinNetIp,
}

pub enum WinNetAddrFamily {
    IPV4,
    IPV6,
}

pub fn get_best_default_route(family: WinNetAddrFamily) -> Result<WinNetDefaultRoute> {
    Ok(WinNetDefaultRoute)
}


fn GetBestDefaultRoute(family: WinNetAddrFamily) -> Result<Option<InterfaceAndGateway>> {

	let mut table: PMIB_IPFORWARD_TABLE2 = PMIB_IPFORWARD_TABLE2::default();

    // TODO: Proper error handling
	unsafe { GetIpForwardTable2(family, &mut table as *mut *mut PMIB_IPFORWARD_TABLE2).map_err(|_| ())? };

    let mut candidates = Vec::with_capacity(table.NumEntries);

	//
	// Enumerate routes looking for: route 0/0
	// The WireGuard interface route has no gateway.
	//

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		const MIB_IPFORWARD_ROW2 &candidate = table->Table[i];

		if (0 == candidate.DestinationPrefix.PrefixLength
			&& RouteHasGateway(candidate)
			&& IsRouteOnPhysicalInterface(candidate))
		{
			candidates.emplace_back(&candidate);
		}
	}

	auto annotated = AnnotateRoutes(candidates);

	if (annotated.empty())
	{
		return std::nullopt;
	}

	//
	// Sort on (active, effectiveMetric) ascending by metric.
	//

	std::sort(annotated.begin(), annotated.end(), [](const AnnotatedRoute &lhs, const AnnotatedRoute &rhs)
	{
		if (lhs.active == rhs.active)
		{
			return lhs.effectiveMetric < rhs.effectiveMetric;
		}

		return lhs.active && false == rhs.active;
	});

	//
	// Ensure the top rated route is active.
	//

	if (false == annotated[0].active)
	{
		return std::nullopt;
	}

	return std::make_optional(InterfaceAndGateway { annotated[0].route->InterfaceLuid, annotated[0].route->NextHop });
}


fn route_has_gateway(route: &MIB_IPFORWARD_ROW2) -> bool {
	match route.NextHop.si_family {
		IpHelper::AF_INET => {
			return 0 != route.NextHop.Ipv4.sin_addr.s_addr;
		}
		IpHelper::AF_INET6 => {
			let sum = &route.NextHop.Ipv6.sin6_addr.u.Byte.iter().sum();
			//const uint8_t *begin = &route.NextHop.Ipv6.sin6_addr.u.Byte[0];
			//const uint8_t *end = begin + 16;

			return 0 != sum;
		}
        _ => {
			return false;
		}
	};
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
