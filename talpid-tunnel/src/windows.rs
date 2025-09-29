use std::io;
use talpid_windows::net::{AddressFamily, get_ip_interface_entry, set_ip_interface_entry};
use windows_sys::Win32::{
    Foundation::ERROR_NOT_FOUND, NetworkManagement::Ndis::NET_LUID_LH,
    Networking::WinSock::RouterDiscoveryDisabled,
};

/// Sets MTU, metric, and disables unnecessary features for the IP interfaces
/// on the specified network interface (identified by `luid`).
pub fn initialize_interfaces(
    luid: NET_LUID_LH,
    ipv4_mtu: Option<u32>,
    ipv6_mtu: Option<u32>,
) -> io::Result<()> {
    for (family, mtu) in &[
        (AddressFamily::Ipv4, ipv4_mtu),
        (AddressFamily::Ipv6, ipv6_mtu),
    ] {
        let mut row = match get_ip_interface_entry(*family, &luid) {
            Ok(row) => row,
            Err(error) if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => continue,
            Err(error) => return Err(error),
        };

        if let Some(mtu) = mtu {
            row.NlMtu = *mtu;
        }

        // Disable DAD, DHCP, and router discovery
        row.SitePrefixLength = 0;
        row.RouterDiscoveryBehavior = RouterDiscoveryDisabled;
        row.DadTransmits = 0;
        row.ManagedAddressConfigurationSupported = false;
        row.OtherStatefulConfigurationSupported = false;

        // Ensure lowest interface metric
        row.Metric = 1;
        row.UseAutomaticMetric = false;

        set_ip_interface_entry(&mut row)?;
    }

    Ok(())
}
