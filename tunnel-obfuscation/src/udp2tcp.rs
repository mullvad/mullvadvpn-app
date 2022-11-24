use crate::Obfuscator;
use async_trait::async_trait;
use std::net::SocketAddr;
use udp_over_tcp::{
    udp2tcp::{self, Udp2Tcp as Udp2TcpImpl},
    TcpOptions,
};

pub struct Udp2TcpSettings {
    pub peer: SocketAddr,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to create obfuscator
    #[error(display = "Failed to create obfuscator")]
    CreateObfuscator(#[error(source)] udp2tcp::Error),

    /// Failed to determine UDP socket details
    #[error(display = "Failed to determine UDP socket details")]
    GetUdpSocketDetails(#[error(source)] std::io::Error),

    /// Failed to run obfuscator
    #[error(display = "Failed to run obfuscator")]
    RunObfuscator(#[error(source)] udp2tcp::Error),
}

struct Udp2Tcp {
    local_addr: SocketAddr,
    instance: Udp2TcpImpl,
}

impl Udp2Tcp {
    pub async fn new(settings: &Udp2TcpSettings) -> Result<Self> {
        let listen_addr = if settings.peer.is_ipv4() {
            SocketAddr::new("127.0.0.1".parse().unwrap(), 0)
        } else {
            SocketAddr::new("::1".parse().unwrap(), 0)
        };

        let instance = Udp2TcpImpl::new(
            listen_addr,
            settings.peer,
            TcpOptions {
                #[cfg(target_os = "linux")]
                fwmark: settings.fwmark,
                ..TcpOptions::default()
            },
        )
        .await
        .map_err(Error::CreateObfuscator)?;
        let local_addr = instance
            .local_udp_addr()
            .map_err(Error::GetUdpSocketDetails)?;

        Ok(Self {
            local_addr,
            instance,
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

    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        self.instance.remote_tcp_fd()
    }
}

pub async fn create_obfuscator(settings: &Udp2TcpSettings) -> Result<Box<dyn Obfuscator>> {
    Ok(Box::new(Udp2Tcp::new(settings).await?))
}
