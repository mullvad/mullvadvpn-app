use std::{
    io,
    net::{IpAddr, SocketAddr},
};

use crate::{Interface, LeakStatus};

use super::{Ip, TracerouteOpt};

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod android;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

/// Implementations that are applicable to all unix platforms.
#[cfg(unix)]
pub mod unix;

/// Implementations that are applicable to all platforms.
pub mod common;

/// Private trait that let's us define the platform-specific implementations and types required for
/// tracerouting.
pub(crate) trait Traceroute {
    type AsyncIcmpSocket: AsyncIcmpSocket;
    type AsyncUdpSocket: AsyncUdpSocket;

    fn get_interface_ip(interface: &Interface, ip_version: Ip) -> anyhow::Result<IpAddr>;

    fn bind_socket_to_interface(
        socket: &socket2::Socket,
        interface: &Interface,
        ip_version: Ip,
    ) -> anyhow::Result<()>;

    /// Configure an ICMP socket to allow reception of ICMP/TimeExceeded errors.
    // TODO: consider moving into AsyncIcmpSocket constructor
    fn configure_icmp_socket(socket: &socket2::Socket, opt: &TracerouteOpt) -> anyhow::Result<()>;
}

pub(crate) trait AsyncIcmpSocket {
    fn from_socket2(socket: socket2::Socket) -> Self;

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()>;

    /// Send an ICMP packet to the destination.
    // TODO: anyhow?
    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize>;

    /// Receive an ICMP packet
    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr)>;

    /// Try to read ICMP/TimeExceeded error packets.
    // TODO: this should be renamed, or not return a LeakStatus
    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus>;
}

pub(crate) trait AsyncUdpSocket {
    fn from_socket2(socket: socket2::Socket) -> Self;

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()>;

    /// Send an UDP packet to the destination.
    // TODO: anyhow?
    async fn send_to(&self, packet: &[u8], destination: impl Into<SocketAddr>)
        -> io::Result<usize>;
}
