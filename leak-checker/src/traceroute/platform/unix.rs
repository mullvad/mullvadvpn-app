use std::net::{IpAddr, SocketAddr};
use std::os::fd::{FromRawFd, IntoRawFd};

use eyre::Context;

use super::AsyncUdpSocket;

pub fn get_interface_ip(interface: &str) -> eyre::Result<IpAddr> {
    for interface_address in nix::ifaddrs::getifaddrs()? {
        if interface_address.interface_name != interface {
            continue;
        };
        let Some(address) = interface_address.address else {
            continue;
        };
        let Some(address) = address.as_sockaddr_in() else {
            continue;
        };

        // TODO: ipv6
        //let Some(address) = address.as_sockaddr_in6() else { continue };

        return Ok(address.ip().into());
    }

    eyre::bail!("Interface {interface:?} has no valid IP to bind to");
}

pub struct AsyncUdpSocketUnix(tokio::net::UdpSocket);

impl AsyncUdpSocket for AsyncUdpSocketUnix {
    fn from_socket2(socket: socket2::Socket) -> Self {
        // HACK: Wrap the socket in a tokio::net::UdpSocket to be able to use it async
        // SAFETY: `into_raw_fd()` consumes the socket and returns an owned & open file descriptor.
        let udp_socket = unsafe { std::net::UdpSocket::from_raw_fd(socket.into_raw_fd()) };
        let udp_socket = tokio::net::UdpSocket::from_std(udp_socket).unwrap();
        AsyncUdpSocketUnix(udp_socket)
    }

    fn set_ttl(&self, ttl: u32) -> eyre::Result<()> {
        self.0
            .set_ttl(ttl)
            .wrap_err("Failed to set TTL value for UDP socket")
    }

    async fn send_to(
        &self,
        packet: &[u8],
        destination: impl Into<SocketAddr>,
    ) -> std::io::Result<usize> {
        self.0.send_to(packet, destination.into()).await
    }
}
