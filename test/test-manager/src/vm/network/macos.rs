use anyhow::{Result, anyhow};
use nix::sys::socket::SockaddrStorage;
use std::net::{Ipv4Addr, SocketAddrV4};

/// Returns the interface name and IP address of the bridge gateway, which is the (first) bridge
/// network that the given `guest_ip` belongs to.
pub(crate) fn find_vm_bridge(guest_ip: &Ipv4Addr) -> Result<(String, Ipv4Addr)> {
    let to_sock_addr = |addr: Option<SockaddrStorage>| {
        addr.as_ref()
            .and_then(|addr| addr.as_sockaddr_in())
            .map(|addr| *SocketAddrV4::from(*addr).ip())
    };

    nix::ifaddrs::getifaddrs()
        .unwrap()
        .filter(|addr| addr.interface_name.starts_with("bridge"))
        .filter_map(|addr| {
            let address = to_sock_addr(addr.address);
            let netmask = to_sock_addr(addr.netmask);
            address
                .zip(netmask)
                .map(|(address, netmask)| (addr.interface_name, address, netmask))
        })
        .find_map(|(interface_name, address, netmask)| {
            ipnetwork::Ipv4Network::with_netmask(address, netmask)
                .ok()
                .filter(|ip_v4_network| ip_v4_network.contains(*guest_ip))
                .map(|_| (interface_name.clone(), address))
        })
        .ok_or_else(|| anyhow!("Failed to identify bridge used by tart -- not running?"))
}
