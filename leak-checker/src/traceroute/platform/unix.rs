use std::net::IpAddr;

pub fn get_interface_ip(interface: &str) -> eyre::Result<IpAddr> {
    for interface_address in nix::ifaddrs::getifaddrs()? {
        if interface_address.interface_name != interface {
            continue;
        };
        let Some(address) = interface_address.address else {
            continue;
        };
        let Some(address) = address.as_sockaddr_in() else {
            continue;
        };

        // TODO: ipv6
        //let Some(address) = address.as_sockaddr_in6() else { continue };

        return Ok(address.ip().into());
    }

    eyre::bail!("Interface {interface:?} has no valid IP to bind to");
}
