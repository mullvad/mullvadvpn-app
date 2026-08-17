use crate::LocalSocketObfuscator;
use async_trait::async_trait;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use talpid_net::bypass::{BypassSocket, SocketBypass};
use tokio::net::{TcpSocket, UdpSocket};
use udp_over_tcp::{HEADER_LEN, process_udp_over_tcp};

#[derive(Debug, Clone)]
pub struct Settings {
    pub peer: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to create the TCP socket
    #[error("Failed to create the TCP socket")]
    CreateTcpSocket(#[source] io::Error),

    /// Failed to disable the Nagle algorithm
    #[error("Failed to disable the Nagle algorithm")]
    SetNodelay(#[source] io::Error),

    /// Failed to bind the local UDP socket
    #[error("Failed to bind the local UDP socket")]
    BindLocalUdp(#[source] io::Error),

    /// Failed to exclude the TCP socket from tunnel traffic
    #[error("Failed to exclude the TCP socket from tunnel traffic")]
    Bypass(#[source] io::Error),

    /// Failed to accept the local WireGuard instance
    #[error("Failed to accept the local WireGuard instance")]
    ConnectLocalUdp(#[source] io::Error),

    /// Failed to connect to the remote
    #[error("Failed to connect to the remote")]
    ConnectTcp(#[source] io::Error),
}

/// Forwards datagrams over a TCP stream, using `udp-over-tcp`.
pub struct Udp2Tcp {
    udp_socket: UdpSocket,
    tcp_socket: BypassSocket<TcpSocket>,
    peer: SocketAddr,
}

impl Udp2Tcp {
    pub(crate) async fn new(
        bypass: Arc<dyn SocketBypass>,
        settings: &Settings,
    ) -> crate::Result<Self> {
        Self::create(bypass, settings)
            .await
            .map_err(crate::Error::CreateUdp2TcpObfuscator)
    }

    async fn create(bypass: Arc<dyn SocketBypass>, settings: &Settings) -> Result<Self, Error> {
        let listen_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
        let udp_socket = UdpSocket::bind(listen_addr)
            .await
            .map_err(Error::BindLocalUdp)?;

        let tcp_socket = match settings.peer {
            SocketAddr::V4(..) => TcpSocket::new_v4(),
            SocketAddr::V6(..) => TcpSocket::new_v6(),
        }
        .map_err(Error::CreateTcpSocket)?;

        // Disables the Nagle algorithm on the TCP socket. Improves performance
        tcp_socket.set_nodelay(true).map_err(Error::SetNodelay)?;

        let tcp_socket = BypassSocket::new(bypass, tcp_socket).map_err(Error::Bypass)?;

        Ok(Self {
            udp_socket,
            tcp_socket,
            peer: settings.peer,
        })
    }

    /// Wait for the local WireGuard instance, then shuttle datagrams until the stream closes.
    async fn forward(self: Box<Self>) -> Result<(), Error> {
        let Self {
            udp_socket,
            tcp_socket,
            peer,
        } = *self;

        let wg_addr = udp_socket
            .peek_sender()
            .await
            .map_err(Error::ConnectLocalUdp)?;
        udp_socket
            .connect(wg_addr)
            .await
            .map_err(Error::ConnectLocalUdp)?;

        let (tcp_socket, _bypass) = (tcp_socket.socket, tcp_socket.guard);
        let tcp_stream = tcp_socket.connect(peer).await.map_err(Error::ConnectTcp)?;
        log::debug!("Connected to {peer}");

        process_udp_over_tcp(udp_socket, tcp_stream, None).await;
        Ok(())
    }
}

#[async_trait]
impl LocalSocketObfuscator for Udp2Tcp {
    fn endpoint(&self) -> SocketAddr {
        self.udp_socket
            .local_addr()
            .expect("the local socket is bound")
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.forward()
            .await
            .map_err(crate::Error::RunUdp2TcpObfuscator)
    }

    fn packet_overhead(&self) -> u16 {
        let max_tcp_header_len = 60; // https://datatracker.ietf.org/doc/html/rfc9293#section-3.1-6.22.1
        let udp_header_len = 8; // https://datatracker.ietf.org/doc/html/rfc768

        let overhead = max_tcp_header_len - udp_header_len + HEADER_LEN;

        u16::try_from(overhead).expect("packet overhead is less than u16::MAX")
    }
}
