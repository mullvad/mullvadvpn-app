use super::{Error, Result};
use std::{io, net::SocketAddr, slice};
use talpid_windows_net::{
    get_ip_interface_entry, try_socketaddr_from_inet_sockaddr, AddressFamily,
};
use widestring::{widecstr, WideCStr};
use windows_sys::Win32::{
    Foundation::NO_ERROR,
    NetworkManagement::{
        IpHelper::{
            FreeMibTable, GetIfEntry2, GetIpForwardTable2, IF_TYPE_SOFTWARE_LOOPBACK,
            IF_TYPE_TUNNEL, MIB_IF_ROW2, MIB_IPFORWARD_ROW2,
        },
        Ndis::NET_LUID_LH,
    },
};

// Interface description substrings found for virtual adapters.
const TUNNEL_INTERFACE_DESCS: [&WideCStr; 3] = [
    widecstr!("WireGuard"),
    widecstr!("Wintun"),
    widecstr!("Tunnel"),
];

fn get_ip_forward_table(family: AddressFamily) -> Result<Vec<MIB_IPFORWARD_ROW2>> {
    let family = family.to_af_family();
    let mut table_ptr = std::ptr::null_mut();

    // SAFETY: GetIpForwardTable2 does not have clear safety specifications however what it does is
    // heap allocate a IpForwardTable2 and then change table_ptr to point to that allocation.
    let status = unsafe { GetIpForwardTable2(family, &mut table_ptr) };
    if NO_ERROR as i32 != status {
        return Err(Error::GetIpForwardTableFailed(
            io::Error::from_raw_os_error(status),
        ));
    }

    // SAFETY: table_ptr is valid since GetIpForwardTable2 did not return an error
    let num_entries = usize::try_from(unsafe { *table_ptr }.NumEntries).unwrap();
    assert!(
        num_entries
            .checked_mul(std::mem::size_of::<MIB_IPFORWARD_ROW2>())
            .unwrap()
            <= usize::try_from(isize::MAX).unwrap()
    );
    // SAFETY: num_entries * size_of(MIB_IPFORWARD_ROW2) is at most isize::MAX
    let rows = unsafe { slice::from_raw_parts((*table_ptr).Table.as_ptr(), num_entries) }.to_vec();

    // SAFETY: FreeMibTable does not have clear safety rules but it deallocates the
    // MIB_IPFORWARD_TABLE2 This pointer is ONLY deallocated here so it is guaranteed to not
    // have been already deallocated. We have cloned all MIB_IPFORWARD_ROW2s and the rows do not
    // contain pointers to the table so they will not be dangling after this free.
    unsafe { FreeMibTable(table_ptr as *const _) }

    Ok(rows)
}

/// General type for passing interface and gateway
pub struct InterfaceAndGateway {
    /// Interface
    pub iface: NET_LUID_LH,
    /// Gateway
    pub gateway: SocketAddr,
}

impl PartialEq for InterfaceAndGateway {
    fn eq(&self, other: &InterfaceAndGateway) -> bool {
        // SAFETY: Accessing Value is always valid in this union as both fields are the same type
        (unsafe { self.iface.Value == other.iface.Value } && self.gateway == other.gateway)
    }
}

/// Get the best default route for the given address family or None if none exists.
pub fn get_best_default_route(family: AddressFamily) -> Result<Option<InterfaceAndGateway>> {
    let table = get_ip_forward_table(family)?;

    // Remove all candidates without a gateway and which are not on a physical interface.
    // Then get the annotated routes which are active.
    let mut annotated: Vec<AnnotatedRoute<'_>> = table
        .iter()
        .filter(|row| {
            0 == row.DestinationPrefix.PrefixLength
                && route_has_gateway(row)
                && is_route_on_physical_interface(row).unwrap_or(false)
        })
        .filter_map(annotate_route)
        .collect();

    // We previously filtered out all inactive routes so we only need to sort by acending
    // effective_metric
    annotated.sort_by(|lhs, rhs| lhs.effective_metric.cmp(&rhs.effective_metric));

    annotated
        .get(0)
        .map(|annotated| {
            Ok(InterfaceAndGateway {
                iface: annotated.route.InterfaceLuid,
                gateway: try_socketaddr_from_inet_sockaddr(annotated.route.NextHop)
                    .map_err(|_| Error::InvalidSiFamily)?,
            })
        })
        .transpose()
}

pub fn route_has_gateway(route: &MIB_IPFORWARD_ROW2) -> bool {
    try_socketaddr_from_inet_sockaddr(route.NextHop)
        .map(|addr| !addr.ip().is_unspecified())
        .unwrap_or(false)
}

// TODO(Jon): It would be more correct to filter for devices that match the known LUID of the tunnel
// interface
fn is_route_on_physical_interface(route: &MIB_IPFORWARD_ROW2) -> Result<bool> {
    // The last 16 bits of _bitfield represent the interface type. For that reason we mask it with
    // 0xFFFF. SAFETY: route.InterfaceLuid is a union. Both variants of this union are always
    // valid since one is a u64 and the other is a wrapped u64. Access to the _bitfield as such
    // is safe since it does not reinterpret the u64 as anything it is not.
    let if_type = u32::try_from(unsafe { route.InterfaceLuid.Info._bitfield } & 0xFFFF).unwrap();
    if if_type == IF_TYPE_SOFTWARE_LOOPBACK || if_type == IF_TYPE_TUNNEL {
        return Ok(false);
    }

    // OpenVPN uses interface type IF_TYPE_PROP_VIRTUAL,
    // but tethering etc. may rely on virtual adapters too,
    // so we have to filter out the TAP adapter specifically.

    // SAFETY: We are allowed to initialize MIB_IF_ROW2 with zeroed because it is made up entirely
    // of types for which the zero pattern (all zeros) is valid.
    let mut row: MIB_IF_ROW2 = unsafe { std::mem::zeroed() };
    row.InterfaceLuid = route.InterfaceLuid;
    row.InterfaceIndex = route.InterfaceIndex;

    // SAFETY: GetIfEntry2 does not have clear safety rules however it will read the
    // row.InterfaceLuid or row.InterfaceIndex and use that information to populate the struct.
    // We guarantee here that these fields are valid since they are set.
    let status = unsafe { GetIfEntry2(&mut row) };
    if NO_ERROR as i32 != status {
        return Err(Error::GetIfEntryFailed(io::Error::from_raw_os_error(
            status,
        )));
    }

    let row_description = WideCStr::from_slice_truncate(&row.Description)
        .expect("Windows provided incorrectly formatted utf16 string");

    for tunnel_interface_desc in TUNNEL_INTERFACE_DESCS {
        if contains_subslice(row_description.as_slice(), tunnel_interface_desc.as_slice()) {
            return Ok(false);
        }
    }

    return Ok(true);
}

fn contains_subslice<T: PartialEq>(slice: &[T], subslice: &[T]) -> bool {
    slice
        .windows(subslice.len())
        .any(|window| window == subslice)
}

struct AnnotatedRoute<'a> {
    route: &'a MIB_IPFORWARD_ROW2,
    effective_metric: u32,
}

fn annotate_route<'a>(route: &'a MIB_IPFORWARD_ROW2) -> Option<AnnotatedRoute<'a>> {
    // SAFETY: `si_family` is valid in both `Ipv4` and `Ipv6` so we can safely access `si_family`.
    let iface = get_ip_interface_entry(
        AddressFamily::try_from_af_family(unsafe { route.DestinationPrefix.Prefix.si_family })
            .ok()?,
        &route.InterfaceLuid,
    )
    .ok()?;

    if iface.Connected == 0 {
        None
    } else {
        Some(AnnotatedRoute {
            route,
            effective_metric: route.Metric + iface.Metric,
        })
    }
}
