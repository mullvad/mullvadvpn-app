use std::net::IpAddr;

use socket2::Socket;

use crate::{
    traceroute::{Ip, TracerouteOpt},
    Interface,
};

use super::{linux, linux::TracerouteLinux, unix, Traceroute};

pub struct TracerouteAndroid;

impl Traceroute for TracerouteAndroid {
    type AsyncIcmpSocket = linux::AsyncIcmpSocketImpl;
    type AsyncUdpSocket = unix::AsyncUdpSocketUnix;

    fn bind_socket_to_interface(
        socket: &Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()> {
        // can't use the same method as desktop-linux here beacuse reasons
        super::common::bind_socket_to_interface::<Self>(socket, interface, ip_version)
    }

    fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr> {
        super::unix::get_interface_ip(interface, ip_version)
    }

    fn configure_icmp_socket(socket: &socket2::Socket, opt: &TracerouteOpt) -> anyhow::Result<()> {
        TracerouteLinux::configure_icmp_socket(socket, opt)
    }
}
