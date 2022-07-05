use crate::windows::{get_ip_interface_entry, set_ip_interface_entry, AddressFamily};
use std::io;
use winapi::shared::{
    ifdef::NET_LUID, nldef::RouterDiscoveryDisabled, ntdef::FALSE, winerror::ERROR_NOT_FOUND,
};

/// Sets MTU, metric, and disables unnecessary features for the IP interfaces
/// on the specified network interface (identified by `luid`).
pub fn initialize_interfaces(luid: NET_LUID, mtu: Option<u32>) -> io::Result<()> {
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
        row.ManagedAddressConfigurationSupported = FALSE;
        row.OtherStatefulConfigurationSupported = FALSE;

        // Ensure lowest interface metric
        row.Metric = 1;
        row.UseAutomaticMetric = FALSE;

        set_ip_interface_entry(&mut row)?;
    }

    Ok(())
}
