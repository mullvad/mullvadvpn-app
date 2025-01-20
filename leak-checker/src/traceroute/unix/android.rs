use socket2::Socket;

use crate::{traceroute::Ip, Interface};

use super::{common::bind_socket_to_interface, linux, Traceroute};

pub struct TracerouteAndroid;

impl Traceroute for TracerouteAndroid {
    type AsyncIcmpSocket = linux::AsyncIcmpSocketImpl;

    fn bind_socket_to_interface(
        socket: &Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()> {
        // can't use the same method as desktop-linux here beacuse reasons
        bind_socket_to_interface(socket, interface, ip_version)
    }
}
