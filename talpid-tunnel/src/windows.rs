use std::io;
use talpid_windows::net::{get_ip_interface_entry, set_ip_interface_entry, AddressFamily};
use windows_sys::Win32::{
    Foundation::ERROR_NOT_FOUND, NetworkManagement::Ndis::NET_LUID_LH,
    Networking::WinSock::RouterDiscoveryDisabled,
};

/// Sets MTU, metric, and disables unnecessary features for the IP interfaces
/// on the specified network interface (identified by `luid`).
pub fn initialize_interfaces(luid: NET_LUID_LH, mtu: Option<u32>) -> io::Result<()> {
    for family in &[AddressFamily::Ipv4, AddressFamily::Ipv6] {
        let mut row = match get_ip_interface_entry(*family, &luid) {
            Ok(row) => row,
            Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => continue,
            Err(error) => return Err(error),
        };

        if let Some(mtu) = mtu {
            row.NlMtu = mtu;
        }

        // Disable DAD, DHCP, and router discovery
        row.SitePrefixLength = 0;
        row.RouterDiscoveryBehavior = RouterDiscoveryDisabled;
        row.DadTransmits = 0;
        row.ManagedAddressConfigurationSupported = 0;
        row.OtherStatefulConfigurationSupported = 0;

        // Ensure lowest interface metric
        row.Metric = 1;
        row.UseAutomaticMetric = 0;

        set_ip_interface_entry(&mut row)?;
    }

    Ok(())
}

/// Sets MTU on the specified network interface for IPv4 and also IPv6 if set.
pub fn set_mtu(iface_name: &str, mtu: u32, use_ipv6: bool) -> io::Result<()> {
    let luid = talpid_windows::net::luid_from_alias(iface_name).map_err(|error| {
        eprint!("Failed to obtain tunnel interface LUID: {}", error);
        error
    })?;

    set_mtu_inner(luid, mtu, use_ipv6)
}

/// Sets MTU on the specified network interface (identified by `luid`).
fn set_mtu_inner(luid: NET_LUID_LH, mtu: u32, use_ipv6: bool) -> io::Result<()> {
    let ip_families: &[AddressFamily] = if use_ipv6 {
        &[AddressFamily::Ipv4, AddressFamily::Ipv6]
    } else {
        &[AddressFamily::Ipv4]
    };
    for family in ip_families {
        let mut row = match get_ip_interface_entry(*family, &luid) {
            Ok(row) => row,
            Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => continue,
            Err(error) => return Err(error),
        };

        row.NlMtu = mtu;

        set_ip_interface_entry(&mut row)?;
    }

    Ok(())
}
