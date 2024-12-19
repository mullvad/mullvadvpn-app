// TODO: Remove this file

use match_cfg::match_cfg;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "android"))]
use std::net::IpAddr;

match_cfg! {
    #[cfg(target_os = "windows")] => {
        pub fn get_interface_ip(interface: &str) -> anyhow::Result<IpAddr> {
            use anyhow::anyhow;

            use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};

            let interface_luid = luid_from_alias(interface)?;

            // TODO: ipv6
            let interface_ip = get_ip_address_for_interface(AddressFamily::Ipv4, interface_luid)?
                .ok_or(anyhow!("No IP for interface {interface:?}"))?;

            Ok(interface_ip)
        }
    }
    #[cfg(any(target_os = "macos", target_os = "android"))] => {
        pub fn get_interface_ip(interface: &str) -> anyhow::Result<IpAddr> {
            for interface_address in nix::ifaddrs::getifaddrs()? {
                if interface_address.interface_name != interface { continue };
                let Some(address) = interface_address.address else { continue };
                let Some(address) = address.as_sockaddr_in() else { continue };
                // TODO: ipv6
                //let Some(address) = address.as_sockaddr_in6() else { continue };

                return Ok(address.ip().into());
            }

            anyhow::bail!("Interface {interface:?} has no valid IP to bind to");
        }
    }
}
