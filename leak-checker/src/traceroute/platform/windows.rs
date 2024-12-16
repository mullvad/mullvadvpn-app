use std::{
    ffi::c_void,
    io, mem,
    net::{IpAddr, SocketAddr},
    os::windows::io::{AsRawSocket, AsSocket, FromRawSocket, IntoRawSocket},
    ptr::null_mut,
};

use anyhow::{bail, anyhow, Context};
use socket2::Socket;
use talpid_windows::net::{get_ip_address_for_interface, luid_from_alias, AddressFamily};

use windows_sys::Win32::Networking::WinSock::{
    WSAGetLastError, WSAIoctl, SIO_RCVALL, SOCKET, SOCKET_ERROR,
};

use crate::{traceroute::TracerouteOpt, LeakStatus};

use super::{common, AsyncIcmpSocket, AsyncUdpSocket, Traceroute};

pub struct TracerouteWindows;

pub struct AsyncIcmpSocketImpl(tokio::net::UdpSocket);

pub struct AsyncUdpSocketWindows(tokio::net::UdpSocket);

impl Traceroute for TracerouteWindows {
    type AsyncIcmpSocket = AsyncIcmpSocketImpl;
    type AsyncUdpSocket = AsyncUdpSocketWindows;

    fn bind_socket_to_interface(socket: &Socket, interface: &str) -> anyhow::Result<()> {
        common::bind_socket_to_interface(socket, interface)
    }

    fn get_interface_ip(interface: &str) -> anyhow::Result<IpAddr> {
        get_interface_ip(interface)
    }

    fn configure_icmp_socket(socket: &socket2::Socket, _opt: &TracerouteOpt) -> anyhow::Result<()> {
        configure_icmp_socket(socket)
    }
}

impl AsyncIcmpSocket for AsyncIcmpSocketImpl {
    fn from_socket2(socket: Socket) -> Self {
        let raw_socket = socket.as_socket().as_raw_socket();
        mem::forget(socket);
        let std_socket = unsafe { std::net::UdpSocket::from_raw_socket(raw_socket) };
        let tokio_socket = tokio::net::UdpSocket::from_std(std_socket).unwrap();
        AsyncIcmpSocketImpl(tokio_socket)
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for ICMP socket")
    }

    async fn send_to(&self, packet: &[u8], destination: impl Into<IpAddr>) -> io::Result<usize> {
        self.0.send_to(packet, (destination.into(), 0)).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, std::net::IpAddr)> {
        let (n, source) = self.0.recv_from(buf).await?;
        Ok((n, source.ip()))
    }

    async fn recv_ttl_responses(&self, opt: &TracerouteOpt) -> anyhow::Result<LeakStatus> {
        common::recv_ttl_responses(self, &opt.interface).await
    }
}

impl AsyncUdpSocket for AsyncUdpSocketWindows {
    fn from_socket2(socket: socket2::Socket) -> Self {
        // HACK: Wrap the socket in a tokio::net::UdpSocket to be able to use it async
        let udp_socket = unsafe { std::net::UdpSocket::from_raw_socket(socket.into_raw_socket()) };
        let udp_socket = tokio::net::UdpSocket::from_std(udp_socket).unwrap();
        AsyncUdpSocketWindows(udp_socket)
    }

    fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        self.0
            .set_ttl(ttl)
            .context("Failed to set TTL value for UDP socket")
    }

    async fn send_to(
        &self,
        packet: &[u8],
        destination: impl Into<SocketAddr>,
    ) -> std::io::Result<usize> {
        self.0.send_to(packet, destination.into()).await
    }
}

pub fn get_interface_ip(interface: &str) -> anyhow::Result<IpAddr> {
    let interface_luid = luid_from_alias(interface)?;

    // TODO: ipv6
    let interface_ip = get_ip_address_for_interface(AddressFamily::Ipv4, interface_luid)?
        .ok_or(anyhow!("No IP for interface {interface:?}"))?;

    Ok(interface_ip)
}

/// Configure the raw socket we use for listening to ICMP responses.
///
/// This will set the `SIO_RCVALL`-option.
pub fn configure_icmp_socket(socket: &Socket) -> anyhow::Result<()> {
    let j = 1;
    let mut _in: u32 = 0;
    let result = unsafe {
        WSAIoctl(
            socket.as_raw_socket() as SOCKET,
            SIO_RCVALL,
            &j as *const _ as *const c_void,
            size_of_val(&j) as u32,
            null_mut(),
            0,
            &mut _in as *mut u32,
            null_mut(),
            None,
        )
    };

    if result == SOCKET_ERROR {
        let code = unsafe { WSAGetLastError() };
        bail!("Failed to call WSAIoctl(listen_socket, SIO_RCVALL, ...), code = {code}");
    }

    Ok(())
}
