use crate::Interface;
use std::net::IpAddr;

/// IP version, v4 or v6, with some associated data.
#[derive(Clone, Copy)]
pub enum Ip<V4 = (), V6 = ()> {
    V4(V4),
    V6(V6),
}

impl Ip {
    pub const fn v4() -> Self {
        Ip::V4(())
    }

    pub const fn v6() -> Self {
        Ip::V6(())
    }
}

#[cfg(target_os = "windows")]
pub fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
    use anyhow::{anyhow, Context};
    use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};

    let interface_luid = match interface {
        Interface::Name(name) => luid_from_alias(name)?,
        Interface::Luid(luid) => *luid,
    };

    let address_family = match ip_version {
        Ip::V4(..) => AddressFamily::Ipv4,
        Ip::V6(..) => AddressFamily::Ipv6,
    };

    get_ip_address_for_interface(address_family, interface_luid)
        .with_context(|| anyhow!("Failed to get IP for interface {interface:?}"))?
        .ok_or(anyhow!("No IP for interface {interface:?}"))
}

#[cfg(unix)]
pub fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
    let Interface::Name(interface) = interface;

    for interface_address in nix::ifaddrs::getifaddrs()? {
        if &interface_address.interface_name != interface {
            continue;
        };
        let Some(address) = interface_address.address else {
            continue;
        };

        match ip_version {
            Ip::V4(()) => {
                if let Some(address) = address.as_sockaddr_in() {
                    return Ok(IpAddr::V4(address.ip()));
                };
            }
            Ip::V6(()) => {
                if let Some(address) = address.as_sockaddr_in6() {
                    return Ok(IpAddr::V6(address.ip()));
                };
            }
        }
    }

    anyhow::bail!("Interface {interface:?} has no valid IP to bind to");
}
