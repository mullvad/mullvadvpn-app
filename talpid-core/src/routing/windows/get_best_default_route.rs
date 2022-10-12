// TODO:
// Verify correctness:
//  Go through the code and make sure that the semantics of everything is the same as C++ or that it is correct - done
//  Do this once before doing all the other changes in the list and then once after
// Restructure project:
//  Go through all 3 modules and split the appropriate functions into their own modules - new module?
//  Go through and rename things that should be renamed - done
//  Go through the code and split things that we repeat >2 times into their own functions - done
//  Go through code and remove unnecessary middle-layer types - done
//  Upgrade RouteManager to be an actor - done
// Correct error handling:
//  Decide what Error type to use and replace everything with that - done
//  Make sure all errors are the same
//  Remove unwraps
//  Log were it is appropriate - done
// Document:
//  Go through and document what should be documented, especially all unsafe code
//  Remove the unnecessary comments - done
// Test:
//  Write down some tests that will be enough to convince you and others that the code is correct
//  Run these tests or write a unit test for the easier ones

use super::{Error, Result};
use crate::windows::{get_ip_interface_entry, try_socketaddr_from_inet_sockaddr, AddressFamily};
use std::{convert::TryInto, io, net::SocketAddr};
use widestring::{widecstr, WideCStr};
use windows_sys::Win32::{
    Foundation::NO_ERROR,
    NetworkManagement::IpHelper::{
        FreeMibTable, GetIfEntry2, GetIpForwardTable2, IF_TYPE_SOFTWARE_LOOPBACK, IF_TYPE_TUNNEL,
        MIB_IF_ROW2, MIB_IPFORWARD_ROW2, NET_LUID_LH,
    },
};

// Interface description substrings found for virtual adapters.
const TUNNEL_INTERFACE_DESCS: [&WideCStr; 3] = [
    widecstr!("WireGuard"),
    widecstr!("Wintun"),
    widecstr!("Tunnel"),
];

fn get_ipforward_rows(family: AddressFamily) -> Result<Vec<MIB_IPFORWARD_ROW2>> {
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
    let num_entries = unsafe { *table_ptr }.NumEntries;
    let mut vec = Vec::with_capacity(num_entries.try_into().unwrap_or_default());

    for i in 0..num_entries {
        assert!(
            usize::try_from(i).unwrap() * std::mem::size_of::<MIB_IPFORWARD_ROW2>()
                < usize::try_from(isize::MAX).unwrap()
        );

        // SAFETY: table_ptr is valid since GetIpForwardTable2 did not return an error nor have we or will we modify the table
        let ptr: *const MIB_IPFORWARD_ROW2 = unsafe { (*table_ptr).Table.as_ptr() };

        // SAFETY: The assert guarantees that the amount of bytes we are jumping is not larger than isize::MAX.
        // Win32 guarantees that the resulting pointer is aligned, non-null, init.
        let row: &MIB_IPFORWARD_ROW2 =
            unsafe { ptr.offset(i.try_into().unwrap()).as_ref() }.unwrap();
        vec.push(row.clone());
    }
    // SAFETY: FreeMibTable does not have clear safety rules but it deallocates the MIB_IPFORWARD_TABLE2
    // This pointer is ONLY deallocated here so it is guaranteed to not have been already deallocated.
    // We have cloned all MIB_IPFORWARD_ROW2s and the rows do not contain pointers to the table so they
    // will not be dangling after this free.
    unsafe { FreeMibTable(table_ptr as *const _) }
    Ok(vec)
}

/// General type for passing interface and gateway
pub struct InterfaceAndGateway {
    //pub iface: NET_LUID_LH,
    /// Interface
    pub iface: NET_LUID_LH,
    //pub gateway: SOCKADDR_INET,
    /// Gateway
    pub gateway: SocketAddr,
}

impl PartialEq for InterfaceAndGateway {
    fn eq(&self, other: &InterfaceAndGateway) -> bool {
        // TODO: Is this OK? We are not comparing the socket address but only comparing the LUID
        unsafe { self.iface.Value == other.iface.Value }
    }
}

/// Get the best default route for the given address family or None if none exists.
pub fn get_best_default_route(family: AddressFamily) -> Result<Option<InterfaceAndGateway>> {
    let table = get_ipforward_rows(family)?;

    // Remove all candidates without a gateway and which are not on a physical interface.
    // Then get the annotated routes which are active.
    let mut annotated: Vec<AnnotatedRoute<'_>> = table
        .iter()
        .filter(|row| {
            0 == row.DestinationPrefix.PrefixLength
                && route_has_gateway(row)
                && is_route_on_physical_interface(row).unwrap_or(false)
        })
        .filter_map(|row| annotate_route(row))
        .collect();

    if annotated.is_empty() {
        return Ok(None);
    }

    // We previously filtered out all inactive routes so we only need to sort by acending effective_metric
    annotated.sort_by(|lhs, rhs| lhs.effective_metric.cmp(&rhs.effective_metric));

    Ok(Some(InterfaceAndGateway {
        iface: annotated[0].route.InterfaceLuid,
        gateway: try_socketaddr_from_inet_sockaddr(annotated[0].route.NextHop)
            .map_err(|_| Error::InvalidSiFamily)?,
    }))
}

pub fn route_has_gateway(route: &MIB_IPFORWARD_ROW2) -> bool {
    match try_socketaddr_from_inet_sockaddr(route.NextHop) {
        Ok(sock) => !sock.ip().is_unspecified(),
        Err(_) => false,
    }
}

// TODO(Jon): It would be more correct to filter for devices that match the known LUID of the tunnel interface
fn is_route_on_physical_interface(route: &MIB_IPFORWARD_ROW2) -> Result<bool> {
    // The last 16 bits of _bitfield represent the interface type. For that reason we mask it with 0xFFFF.
    // SAFETY: route.InterfaceLuid is a union. Both variants of this union are always valid since one is a u64
    // and the other is a wrapped u64. Access to the _bitfield as such is safe since it does not reinterpret the
    // u64 as anything it is not.
    let if_type = u32::try_from(unsafe { route.InterfaceLuid.Info._bitfield } & 0xFFFF).unwrap();
    if if_type == IF_TYPE_SOFTWARE_LOOPBACK || if_type == IF_TYPE_TUNNEL {
        return Ok(false);
    }

    // OpenVPN uses interface type IF_TYPE_PROP_VIRTUAL,
    // but tethering etc. may rely on virtual adapters too,
    // so we have to filter out the TAP adapter specifically.

    // SAFETY: We are allowed to initialize MIB_IF_ROW2 with zeroed because it is made up entirely of types for which the
    // zero pattern (all zeros) is valid.
    let mut row: MIB_IF_ROW2 = unsafe { std::mem::zeroed() };
    row.InterfaceLuid = route.InterfaceLuid;
    row.InterfaceIndex = route.InterfaceIndex;

    // SAFETY: GetIfEntry2 does not have clear safety rules however it will read the row.InterfaceLuid or row.InterfaceIndex and use
    // that information to populate the struct. We guarantee here that these fields are valid since they are set.
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
