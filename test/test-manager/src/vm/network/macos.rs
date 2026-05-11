use anyhow::{Context, Result, anyhow};
use gotatun::device::{DefaultDeviceTransports, Device};
use nix::sys::socket::SockaddrStorage;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::process::Command;

use crate::vm::network::wireguard;

/// Set up WireGuard relay and dummy hosts.
pub async fn setup_test_network() -> Result<Device<DefaultDeviceTransports>> {
    log::debug!("Setting up test network");

    enable_forwarding().await?;
    wireguard::create_interface()
        .await
        .context("Failed to create WireGuard interface")
}

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

async fn enable_forwarding() -> Result<()> {
    // Enable forwarding
    let mut cmd = Command::new("/usr/bin/sudo");
    cmd.args(["/usr/sbin/sysctl", "net.inet.ip.forwarding=1"]);
    let output = cmd.output().await.context("Run sysctl")?;
    if !output.status.success() {
        return Err(anyhow!("sysctl failed: {}", output.status.code().unwrap()));
    }
    Ok(())
}
