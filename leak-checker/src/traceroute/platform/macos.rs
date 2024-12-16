use std::io;
use std::net::IpAddr;
use std::os::fd::{FromRawFd, IntoRawFd};

use anyhow::Context;
use socket2::Socket;

use crate::traceroute::TracerouteOpt;
use crate::LeakStatus;

use super::{common, unix, AsyncIcmpSocket, Traceroute};

pub struct TracerouteMacos;

pub struct AsyncIcmpSocketImpl(tokio::net::UdpSocket);

impl Traceroute for TracerouteMacos {
    type AsyncIcmpSocket = AsyncIcmpSocketImpl;
    type AsyncUdpSocket = unix::AsyncUdpSocketUnix;

    fn bind_socket_to_interface(socket: &Socket, interface: &str) -> anyhow::Result<()> {
        // can't use the same method as desktop-linux here beacuse reasons
        bind_socket_to_interface(socket, interface)
    }

    fn get_interface_ip(interface: &str) -> anyhow::Result<IpAddr> {
        super::unix::get_interface_ip(interface)
    }

    fn configure_icmp_socket(
        _socket: &socket2::Socket,
        _opt: &TracerouteOpt,
    ) -> anyhow::Result<()> {
        Ok(())
        // TODO: not sure if we need to do anything here
    }
}

impl AsyncIcmpSocket for AsyncIcmpSocketImpl {
    fn from_socket2(socket: Socket) -> Self {
        let raw_socket = socket.into_raw_fd();
        let std_socket = unsafe { std::net::UdpSocket::from_raw_fd(raw_socket) };
        let tokio_socket = tokio::net::UdpSocket::from_std(std_socket).unwrap();
        AsyncIcmpSocketImpl(tokio_socket)
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for socket")
    }

    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize> {
        self.0.send_to(packet, (destination.into(), 0)).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, IpAddr)> {
        self.0
            .recv_from(buf)
            .await
            .map(|(n, source)| (n, source.ip()))
    }

    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
        common::recv_ttl_responses(self, &opt.interface).await
    }
}

pub fn bind_socket_to_interface(socket: &Socket, interface: &str) -> anyhow::Result<()> {
    use nix::net::if_::if_nametoindex;
    use std::num::NonZero;

    log::info!("Binding socket to {interface:?}");

    let interface_index = if_nametoindex(interface)
        .map_err(anyhow::Report::from)
        .and_then(|code| NonZero::new(code).ok_or_anyhow("Non-zero error code"))
        .context("Failed to get interface index")?;

    socket.bind_device_by_index_v4(Some(interface_index))?;
    Ok(())
}
