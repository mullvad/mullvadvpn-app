use async_trait::async_trait;
use std::{
    iter,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use talpid_types::net::{
    obfuscation::{ObfuscatorConfig, Obfuscators},
    wireguard::PublicKey,
};
use tokio::io;

pub mod lwo;
pub mod multiplexer;
pub mod quic;
pub mod shadowsocks;
pub mod socket;
pub mod udp2tcp;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create Udp2Tcp obfuscator")]
    CreateUdp2TcpObfuscator(#[source] udp2tcp::Error),

    #[error("Failed to run Udp2Tcp obfuscator")]
    RunUdp2TcpObfuscator(#[source] udp2tcp::Error),

    #[error("Failed to initialize Shadowsocks")]
    CreateShadowsocksObfuscator(#[source] shadowsocks::Error),

    #[error("Failed to run Shadowsocks")]
    RunShadowsocksObfuscator(#[source] shadowsocks::Error),

    #[error("Failed to initialize Quic")]
    CreateQuicObfuscator(#[source] quic::Error),

    #[error("Failed to run Quic")]
    RunQuicObfuscator(#[source] quic::Error),

    #[error("Failed to initialize LWO")]
    CreateLwoObfuscator(#[source] lwo::Error),

    #[error("Failed to run LWO")]
    RunLwoObfuscator(#[source] lwo::Error),

    #[error(transparent)]
    CreateSocket(#[from] socket::Error),

    #[error("Failed to initialize multiplexer")]
    CreateMultiplexerObfuscator(#[source] io::Error),

    #[error("Failed to run multiplexer")]
    RunMultiplexerObfuscator(#[source] io::Error),
}

#[async_trait]
pub trait LocalSocketObfuscator: Send {
    /// NOTE(Android): Make sure to call bypass on the obfuscator socket _before_ invoking run.
    async fn run(self: Box<Self>) -> Result<()>;

    /// Returns the address of the local socket.
    fn endpoint(&self) -> SocketAddr;

    /// Returns the file descriptor of the outbound socket.
    #[cfg(target_os = "android")]
    fn remote_socket_fd(&self) -> std::os::unix::io::RawFd;

    /// The overhead (in bytes) of this obfuscation protocol.
    ///
    /// This is used when deciding on MTUs.
    fn packet_overhead(&self) -> u16;
}

#[derive(Debug, Clone)]
pub enum Settings {
    Udp2Tcp(udp2tcp::Settings),
    Shadowsocks(shadowsocks::Settings),
    Quic(quic::Settings),
    Lwo(lwo::Settings),
    Multiplexer(multiplexer::Settings),
}

impl Settings {
    pub fn new(
        client_public_key: PublicKey,
        server_public_key: PublicKey,
        obfuscation_config: &Obfuscators,
        mtu: u16,
        #[cfg(target_os = "linux")] fwmark: Option<u32>,
    ) -> Self {
        match obfuscation_config {
            Obfuscators::Single(obfuscation_config) => settings_from_single_config(
                client_public_key,
                server_public_key,
                obfuscation_config,
                mtu,
                #[cfg(target_os = "linux")]
                fwmark,
            ),
            Obfuscators::Multiplexer {
                direct,
                configs: (first_obfs, remaining_obfs),
            } => {
                let mut transports = vec![];
                if let Some(direct) = direct {
                    transports.push(multiplexer::Transport::Direct(*direct));
                }
                for obfs_config in iter::once(first_obfs).chain(remaining_obfs) {
                    let settings = settings_from_single_config(
                        client_public_key.clone(),
                        server_public_key.clone(),
                        obfs_config,
                        mtu,
                        #[cfg(target_os = "linux")]
                        fwmark,
                    );
                    transports.push(multiplexer::Transport::Obfuscated(settings));
                }
                Self::Multiplexer(multiplexer::Settings {
                    transports,
                    #[cfg(target_os = "linux")]
                    fwmark,
                })
            }
        }
    }
}
pub fn settings_from_single_config(
    client_public_key: PublicKey,
    server_public_key: PublicKey,
    obfuscation_config: &ObfuscatorConfig,
    mtu: u16,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Settings {
    match obfuscation_config {
        ObfuscatorConfig::Udp2Tcp { endpoint } => Settings::Udp2Tcp(udp2tcp::Settings {
            peer: *endpoint,
            #[cfg(target_os = "linux")]
            fwmark,
        }),
        ObfuscatorConfig::Shadowsocks { endpoint } => {
            Settings::Shadowsocks(shadowsocks::Settings {
                shadowsocks_endpoint: *endpoint,
                wireguard_endpoint: if endpoint.is_ipv4() {
                    SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
                } else {
                    SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
                },
                #[cfg(target_os = "linux")]
                fwmark,
            })
        }
        ObfuscatorConfig::Quic {
            hostname,
            endpoint,
            auth_token,
        } => {
            let wireguard_endpoint = SocketAddr::from((Ipv4Addr::LOCALHOST, 51820));
            let settings = quic::Settings::new(
                *endpoint,
                hostname.to_owned(),
                auth_token.parse().unwrap(),
                wireguard_endpoint,
            )
            .mtu(mtu);
            #[cfg(target_os = "linux")]
            if let Some(fwmark) = fwmark {
                let mut settings = settings;
                settings.set_fwmark(fwmark);
                return Settings::Quic(settings);
            }
            Settings::Quic(settings)
        }
        ObfuscatorConfig::Lwo { endpoint } => Settings::Lwo(lwo::Settings {
            server_addr: *endpoint,
            client_public_key,
            server_public_key,
            #[cfg(target_os = "linux")]
            fwmark,
        }),
    }
}

pub async fn create_local_socket_obfuscator(
    settings: &Settings,
) -> Result<Box<dyn LocalSocketObfuscator>> {
    match settings {
        Settings::Udp2Tcp(s) => udp2tcp::Udp2Tcp::new(s)
            .await
            .map(box_obfuscator)
            .map_err(Error::CreateUdp2TcpObfuscator),
        Settings::Shadowsocks(s) => shadowsocks::Shadowsocks::new(s).await.map(box_obfuscator),
        Settings::Quic(s) => quic::QuicLocalSocket::new(s).await.map(box_obfuscator),
        Settings::Lwo(s) => lwo::Lwo::new(s).await.map(box_obfuscator),
        Settings::Multiplexer(s) => multiplexer::Multiplexer::new(s).await.map(box_obfuscator),
    }
}

fn box_obfuscator(obfs: impl LocalSocketObfuscator + 'static) -> Box<dyn LocalSocketObfuscator> {
    Box::new(obfs) as Box<dyn LocalSocketObfuscator>
}
