use super::Traceroute;
use crate::Interface;
use crate::util::{Ip, get_interface_ip};
use anyhow::Context;
use socket2::Socket;
use std::net::SocketAddr;

pub struct TracerouteAndroid;

impl Traceroute for TracerouteAndroid {
    type AsyncIcmpSocket = super::linux_like::AsyncIcmpSocketImpl;

    fn bind_socket_to_interface(
        socket: &Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()> {
        // We do not have permission to bind directly to an interface on Android,
        // unlike desktop Linux. Therefore we bind to the interface IP instead.
        bind_socket_to_interface(socket, interface, ip_version)
    }
}

fn bind_socket_to_interface(
    socket: &Socket,
    interface: &Interface,
    ip_version: Ip,
) -> anyhow::Result<()> {
    let interface_ip = get_interface_ip(interface, ip_version)?;

    log::debug!("Binding socket to {interface_ip} ({interface:?})");

    socket
        .bind(&SocketAddr::new(interface_ip, 0).into())
        .context("Failed to bind socket to interface address")?;

    Ok(())
}
