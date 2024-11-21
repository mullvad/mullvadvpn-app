use std::net::IpAddr;

use socket2::Socket;

use crate::traceroute::TracerouteOpt;

use super::{linux, linux::TracerouteLinux, unix, Traceroute};

pub struct TracerouteAndroid;

impl Traceroute for TracerouteAndroid {
    type AsyncIcmpSocket = linux::AsyncIcmpSocketImpl;
    type AsyncUdpSocket = unix::AsyncUdpSocketUnix;

    fn bind_socket_to_interface(socket: &Socket, interface: &str) -> eyre::Result<()> {
        // can't use the same method as desktop-linux here beacuse reasons
        super::common::bind_socket_to_interface(socket, interface)
    }

    fn get_interface_ip(interface: &str) -> eyre::Result<IpAddr> {
        super::unix::get_interface_ip(interface)
    }

    fn configure_icmp_socket(socket: &socket2::Socket, opt: &TracerouteOpt) -> eyre::Result<()> {
        TracerouteLinux::configure_icmp_socket(socket, opt)
    }
}
