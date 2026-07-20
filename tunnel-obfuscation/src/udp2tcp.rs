use crate::Obfuscator;
use async_trait::async_trait;
use std::{net::SocketAddr, sync::Arc};
use talpid_net::bypass::{BypassGuard, SocketBypass};
use udp_over_tcp::{
    TcpOptions,
    udp2tcp::{self, Udp2Tcp as Udp2TcpImpl},
};

#[derive(Debug, Clone)]
pub struct Settings {
    pub peer: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to create obfuscator
    #[error("Failed to create obfuscator")]
    CreateObfuscator(#[source] udp2tcp::Error),

    /// Failed to determine UDP socket details
    #[error("Failed to determine UDP socket details")]
    GetUdpSocketDetails(#[source] std::io::Error),

    /// Failed to run obfuscator
    #[error("Failed to run obfuscator")]
    RunObfuscator(#[source] udp2tcp::Error),
}

pub struct Udp2Tcp {
    local_addr: SocketAddr,
    instance: Udp2TcpImpl,
    _bypass: BypassGuard,
}

impl Udp2Tcp {
    pub(crate) async fn new(
        bypass: Arc<dyn SocketBypass>,
        settings: &Settings,
    ) -> crate::Result<Self> {
        let listen_addr = if settings.peer.is_ipv4() {
            SocketAddr::new("127.0.0.1".parse().unwrap(), 0)
        } else {
            SocketAddr::new("::1".parse().unwrap(), 0)
        };

        let mut tcp_options = TcpOptions::default();
        // Disables the Nagle algorithm on the TCP socket. Improves performance
        tcp_options.nodelay = true;

        let instance = Udp2TcpImpl::new(listen_addr, settings.peer, tcp_options)
            .await
            .map_err(Error::CreateObfuscator)
            .map_err(crate::Error::CreateUdp2TcpObfuscator)?;
        let local_addr = instance
            .local_udp_addr()
            .map_err(Error::GetUdpSocketDetails)
            .map_err(crate::Error::CreateUdp2TcpObfuscator)?;

        cfg_select! {
            unix => {
                use std::os::fd::BorrowedFd;

                // SAFETY: The fd is a valid socket and valid for the lifetime of instance
                let fd = unsafe { BorrowedFd::borrow_raw(instance.remote_tcp_fd()) };

                let _bypass = BypassGuard::new(bypass, &fd).map_err(crate::Error::Bypass)?;
            }
            windows => {
                use std::os::windows::io::BorrowedSocket;

                // SAFETY: This is a valid socket and valid for the lifetime of instance
                let sock = unsafe { BorrowedSocket::borrow_raw(instance.remote_tcp_socket()) };

                let _bypass = BypassGuard::new(bypass, &sock).map_err(crate::Error::Bypass)?;
            }
            _ => unimplemented!("unsupported OS")
        }

        Ok(Self {
            local_addr,
            instance,
            _bypass,
        })
    }
}

#[async_trait]
impl Obfuscator for Udp2Tcp {
    fn endpoint(&self) -> SocketAddr {
        self.local_addr
    }

    async fn run(self: Box<Self>) -> crate::Result<()> {
        self.instance
            .run()
            .await
            .map_err(Error::RunObfuscator)
            .map_err(crate::Error::RunUdp2TcpObfuscator)
    }

    fn packet_overhead(&self) -> u16 {
        let max_tcp_header_len = 60; // https://datatracker.ietf.org/doc/html/rfc9293#section-3.1-6.22.1
        let udp_header_len = 8; // https://datatracker.ietf.org/doc/html/rfc768

        // TODO: Make `HEADER_LEN` constant public in udp-over-tcp lib and use it instead
        let udp_over_tcp_header_len = size_of::<u16>();

        let overhead = max_tcp_header_len - udp_header_len + udp_over_tcp_header_len;

        u16::try_from(overhead).expect("packet overhead is less than u16::MAX")
    }
}
