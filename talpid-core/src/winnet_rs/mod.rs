use std::convert::TryInto;
use windows::Win32::
{
    NetworkManagement::IpHelper::{GetIpForwardTable2, MIB_IPFORWARD_TABLE2, MIB_IPFORWARD_ROW2, MIB_IF_ROW2, FreeMibTable, GetIpInterfaceEntry, NET_LUID_LH, IF_TYPE_SOFTWARE_LOOPBACK, IF_TYPE_TUNNEL, MIB_IPINTERFACE_ROW, GetIfEntry2},
    Networking::WinSock::{AF_INET, AF_INET6, SOCKADDR_INET}
};
use widestring::U16CString;

// Interface description substrings found for virtual adapters.
const TUNNEL_INTERFACE_DESCS: [&str; 3] = [
	"WireGuard",
	"Wintun",
	"Tunnel"
];

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// The si family that windows should provide should be either Ipv4 or Ipv6. This is a serious bug and might become a panic.
    #[error(display = "The si family provided by windows is incorrect")]
    InvalidSiFamily,
    /// Converion error between types that should not be possible. Indicates serious error and might become a panic.
    #[error(display = "Conversion between types provided by windows failed")]
    Conversion,
    /// A windows API failed
    #[error(display = "Windows API call failed")]
    WindowsApi,
}

type Result<T> = std::result::Result<T, Error>;

pub enum WinNetIp {
    IPV4([u8; 4]),
    IPV6([u8; 16])
}

pub struct WinNetDefaultRoute {
    pub interface_luid: u64,
    pub gateway: WinNetIp,
}

#[derive(Debug)]
pub enum WinNetAddrFamily {
    IPV4,
    IPV6,
}


fn ip_from_native(from: &SOCKADDR_INET) -> Result<WinNetIp> {
    dbg!(& unsafe {from.si_family});
    // SAFETY: `si_family` is valid in both `Ipv4` and `Ipv6` so we can safely access `si_family`.
    if u32::from(unsafe { from.si_family }) == AF_INET.0 {
        // SAFETY: `Ipv4` is valid since `si_family` specifies that.
        // `S_addr` is another union field but it is always valid if
        // the Ipv4 representation is valid so access is safe.
        let u32_addr = unsafe { from.Ipv4.sin_addr.S_un.S_addr };
        // Must convert S_addr to big-endian in order to have the correct network byte order
        let u32_addr = u32_addr.to_be();
        Ok(WinNetIp::IPV4(u32_addr.to_be_bytes()))
    } else if u32::from(unsafe { from.si_family } ) == AF_INET6.0 {
        // SAFETY: `Ipv6` is valid since `si_family` specifies that.
        // `Byte` is another union field but this one is always valid if
        // the Ipv6 representation is valid so access is safe.
        Ok(WinNetIp::IPV6(unsafe { from.Ipv6.sin6_addr.u.Byte }.clone()))
    } else {
        Err(Error::InvalidSiFamily)
    }
}

pub fn get_best_default_route(family: WinNetAddrFamily) -> Result<Option<WinNetDefaultRoute>> {
    match get_best_default_internal(family)? {
        Some(iface_and_gateway) => {
            Ok(Some(WinNetDefaultRoute {
                // SAFETY: both fields in the NET_LUID_LH union are ultimately u64 and as such both will be valid and safe to access.
                interface_luid: unsafe { iface_and_gateway.iface.Value },
                gateway: ip_from_native(&iface_and_gateway.gateway)?,
            }))
        }
        None => Ok(None)
    }
}

struct MibIpforwardTable2(*mut MIB_IPFORWARD_TABLE2);

impl MibIpforwardTable2 {
    fn new(family: WinNetAddrFamily) -> Result<Self> {
        let family = match family {
            WinNetAddrFamily::IPV4 => AF_INET,
            WinNetAddrFamily::IPV6 => AF_INET6,
        };
        let mut table_ptr = std::ptr::null_mut();
        // SAFETY: GetIpForwardTable2 does not have clear safety specifications however what it does is
        // heap allocate a IpForwardTable2 and then change table_ptr to point to that allocation.
        unsafe { GetIpForwardTable2(u16::try_from(family.0).map_err(|_| Error::Conversion)?, &mut table_ptr).map_err(|_| Error::WindowsApi)? };
        Ok(Self(table_ptr))
    }

    fn get_table_entry(&self, i: u32) -> &MIB_IPFORWARD_ROW2 {
        assert!(i < self.num_entries() );
        assert!(usize::try_from(i).unwrap() * std::mem::size_of::<MIB_IPFORWARD_ROW2>() < usize::try_from(isize::MAX).unwrap());

        // SAFETY: The slice will live as long as safe is not dropped. As such this pointer
        // is guaranteed not to point to garbage. It is also ensured that the slice is not modified
        // as we are tying this to a &'a of self.
        let ptr: *const MIB_IPFORWARD_ROW2 = unsafe { (*self.0).Table.as_ptr() };
        // SAFETY: The first assert guarantees that i does not refer to an out-of-bounds.
        // The second assert guarantees that i is not larger than isize::MAX.
        // Win32 guarantees that the resulting pointer is aligned, non-null, init.
        // The underlying pointer will not be freed until self is dropped, which guarantees that the reference lives
        // long enough.
        let row: &MIB_IPFORWARD_ROW2 = unsafe { ptr.offset(i.try_into().unwrap()).as_ref() }.unwrap();
        row
    }

    fn num_entries(&self) -> u32 {
        // SAFETY: self.0 is always valid since the MIB_IPFORWARD_TABLE2 is allocated by `Self::new()` and is deallocated on drop.
        // self.0 is guaranteed to not be mutably accessed somewhere else since `&self` is taken.
        unsafe { *self.0 }.NumEntries
    }
}

impl Drop for MibIpforwardTable2 {
    fn drop(&mut self) {
        // SAFETY: FreeMibTable does not have clear safety rules but it deallocates the MIB_IPFORWARD_TABLE2
        // This pointer will not be accessed after this since this is drop.
        // This pointer is ONLY deallocated here so it is guaranteed to not have been already deallocated.
        unsafe { FreeMibTable(self.0 as *const _) }
    }
}

struct InterfaceAndGateway {
    iface: NET_LUID_LH,
    gateway: SOCKADDR_INET,
}

fn get_best_default_internal(family: WinNetAddrFamily) -> Result<Option<InterfaceAndGateway>> {
    dbg!(&family);
    let table = MibIpforwardTable2::new(family)?;
    let mut candidates = Vec::with_capacity(usize::try_from(table.num_entries()).map_err(|_| Error::Conversion)?);

	//
	// Enumerate routes looking for: route 0/0
	// The WireGuard interface route has no gateway.
	//

    for i in 0..table.num_entries() {
		let candidate = table.get_table_entry(i);

		if 0 == candidate.DestinationPrefix.PrefixLength
			&& route_has_gateway(candidate)
			&& is_route_on_physical_interface(candidate)?
		{
			candidates.push(candidate);
		}
	}

	let mut annotated = annotated_routes(&candidates);

	if annotated.is_empty() {
		return Ok(None);
	}

	//
	// Sort on (active, effectiveMetric) ascending by metric.
	//

    annotated.sort_by(|lhs, rhs| {
        if lhs.active == rhs.active {
            return lhs.effective_metric.cmp(&rhs.effective_metric);
        }
        //return lhs.active && false == rhs.active;
        if lhs.active {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

	//
	// Ensure the top rated route is active.
	//

	if false == annotated[0].active {
		return Ok(None);
	}

	Ok(Some(InterfaceAndGateway { iface: annotated[0].route.InterfaceLuid, gateway: annotated[0].route.NextHop }))
}


fn route_has_gateway(route: &MIB_IPFORWARD_ROW2) -> bool {
    match ip_from_native(&route.NextHop) {
        Ok(WinNetIp::IPV4(ipv4)) => {
            let sum: u32 = ipv4.iter().map(|b| u32::from(*b)).sum();
            0 != sum
        }
        Ok(WinNetIp::IPV6(ipv6)) => {
            let sum: u32 = ipv6.iter().map(|b| u32::from(*b)).sum();
            0 != sum
        }
        Err(_) => false
    }
}


fn is_route_on_physical_interface(route: &MIB_IPFORWARD_ROW2) -> Result<bool> {
    // FIXME: TEST THIS TO MAKE SURE IT WORKS

    // The last 16 bits of _bitfield represent the interface type. For that reason we mask it with 0xFFFF.
    // SAFETY: route.InterfaceLuid is a union. Both variants of this union are always valid since one is a u64
    // and the other is a wrapped u64. Access to the _bitfield as such is safe since it does not reinterpret the 
    // u64 as anything it is not.
    let if_type = unsafe { route.InterfaceLuid.Info._bitfield } & 0xFFFF;
    if if_type == u64::from(IF_TYPE_SOFTWARE_LOOPBACK) ||
        if_type == u64::from(IF_TYPE_TUNNEL) {
        return Ok(false);
    }
    
    // OpenVPN uses interface type IF_TYPE_PROP_VIRTUAL,
    // but tethering etc. may rely on virtual adapters too,
    // so we have to filter out the TAP adapter specifically.
    
    let mut row = MIB_IF_ROW2::default();
    row.InterfaceLuid = route.InterfaceLuid;
    row.InterfaceIndex = route.InterfaceIndex;
    
    // SAFETY: GetIfEntry2 does not have clear safety rules however it will read the row.InterfaceLuid or row.InterfaceIndex and use
    // that information to populate the struct. We guarantee here that this struct and these fields are valid since they are initliazed
    // through default.
    unsafe { GetIfEntry2(&mut row as *mut MIB_IF_ROW2) }.map_err(|_| Error::WindowsApi)?;

    let row_description =
        U16CString::from_vec_truncate(row.Description)
            .to_string()
            .expect("Windows provided incorrectly formatted utf16 string");
    for tunnel_interface_desc in TUNNEL_INTERFACE_DESCS {
    	if row_description.contains(tunnel_interface_desc) {
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

        // SAFETY: `si_family` is valid in both `Ipv4` and `Ipv6` so we can safely access `si_family`.
        iface.Family = unsafe { route.DestinationPrefix.Prefix.si_family };
        iface.InterfaceLuid = route.InterfaceLuid;

        // SAFETY: GetIpInterfaceEntry does not have clear safety rules however GetIpInterfaceEntry will read the iface.InterfaceLuid
        // or iface.InterfaceIndex and use that information to populate the struct. We guarantee here that this struct and these
        // fields are valid since they are initliazed through default.
        match unsafe { GetIpInterfaceEntry(&mut iface as *mut MIB_IPINTERFACE_ROW) } {
            Ok(()) => {
                Some(AnnotatedRoute {
                    route,
                    active: iface.Connected.0 != 0,
                    effective_metric: route.Metric + iface.Metric,
                })
            },
            Err(_) => {
                None
            }
        }
    }).collect()
}
