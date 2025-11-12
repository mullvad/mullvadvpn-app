use anyhow::Context;
use socket2::Socket;

use crate::{Interface, util::Ip};

pub struct TracerouteLinux;

impl super::Traceroute for TracerouteLinux {
    type AsyncIcmpSocket = super::linux_like::AsyncIcmpSocketImpl;

    fn bind_socket_to_interface(
        socket: &Socket,
        interface: &Interface,
        _: Ip,
    ) -> anyhow::Result<()> {
        bind_socket_to_interface(socket, interface)
    }
}

fn bind_socket_to_interface(socket: &Socket, interface: &Interface) -> anyhow::Result<()> {
    log::debug!("Binding socket to {interface:?}");

    let Interface::Name(interface) = interface;

    socket
        .bind_device(Some(interface.as_bytes()))
        .context("Failed to bind socket to interface")?;

    Ok(())
}
