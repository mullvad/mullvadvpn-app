use socket2::Socket;

use crate::{util::Ip, Interface};

use super::{common::bind_socket_to_interface, linux, Traceroute};

pub struct TracerouteAndroid;

impl Traceroute for TracerouteAndroid {
    type AsyncIcmpSocket = linux::AsyncIcmpSocketImpl;

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
