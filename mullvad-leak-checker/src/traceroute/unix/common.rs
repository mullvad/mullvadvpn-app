#![allow(dead_code)] // some code here is not used on some targets.

use std::net::SocketAddr;

use anyhow::Context;
use socket2::Socket;

use crate::util::{get_interface_ip, Ip};
use crate::Interface;

pub(crate) fn bind_socket_to_interface(
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
