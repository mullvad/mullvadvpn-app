use socket2::Socket;

use crate::{
    traceroute::{Ip, TracerouteOpt},
    Interface,
};

use super::{
    common::bind_socket_to_interface,
    linux::{self, TracerouteLinux},
    Traceroute,
};

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

    fn configure_icmp_socket(socket: &socket2::Socket, opt: &TracerouteOpt) -> anyhow::Result<()> {
        TracerouteLinux::configure_icmp_socket(socket, opt)
    }
}
