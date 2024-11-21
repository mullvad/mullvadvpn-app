use std::{io, net::IpAddr};

use crate::LeakStatus;

use super::TracerouteOpt;

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
pub trait Traceroute {
    type AsyncIcmpSocket: AsyncIcmpSocket;

    fn get_interface_ip(interface: &str) -> eyre::Result<IpAddr>;

    fn bind_socket_to_interface(socket: &socket2::Socket, interface: &str) -> eyre::Result<()>;

    fn configure_icmp_socket(socket: &socket2::Socket, opt: &TracerouteOpt) -> eyre::Result<()>;
}

pub trait AsyncIcmpSocket {
    fn from_socket2(socket: socket2::Socket) -> Self;
    fn set_ttl(&self, ttl: u32) -> eyre::Result<()>;

    /// Send an icmp packet to the destination.
    // TODO: eyre?
    async fn send_to(&self, packet: &[u8], destination: IpAddr) -> io::Result<usize>;

    /// Receive an icmp packet
    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr)>;

    /// Try to read ICMP/TimeExceeded error packets.
    // TODO: this should be renamed, or not return a LeakStatus
    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> eyre::Result<LeakStatus>;
}

#[cfg(target_os = "android")]
pub type Impl = platform::android::TracerouteAndroid;

#[cfg(target_os = "linux")]
pub type Impl = linux::TracerouteLinux;

#[cfg(target_os = "macos")]
pub type Impl = macos::TracerouteMacos;

#[cfg(target_os = "windows")]
pub type Impl = windows::TracerouteWindows;

pub type AsyncIcmpSocketImpl = <Impl as Traceroute>::AsyncIcmpSocket;
