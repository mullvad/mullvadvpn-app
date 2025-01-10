#![allow(dead_code)] // some code here is not used on some targets.

use std::net::{IpAddr, SocketAddr};

use anyhow::Context;
use socket2::Socket;

use crate::{traceroute::Ip, Interface};

pub(crate) fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
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

pub(crate) fn bind_socket_to_interface(
    socket: &Socket,
    interface: &Interface,
    ip_version: Ip,
) -> anyhow::Result<()> {
    let interface_ip = get_interface_ip(interface, ip_version)?;

    log::info!("Binding socket to {interface_ip} ({interface:?})");

    socket
        .bind(&SocketAddr::new(interface_ip, 0).into())
        .context("Failed to bind socket to interface address")?;

    Ok(())
}
