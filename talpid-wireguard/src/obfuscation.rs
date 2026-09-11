//! Glue between tunnel-obfuscation and WireGuard configurations

use super::{Error, Result};
use std::{
    iter,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};
use talpid_net::bypass::{BypassToken, SocketBypass};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::TunProvider;
use talpid_types::net::{
    obfuscation::{ObfuscatorConfig, Obfuscators},
    wireguard::PublicKey,
};
use tokio::sync::oneshot;
use tunnel_obfuscation::{
    ObfuscatedTransport, create_transport, lwo,
    multiplexer::{self, Multiplexer, Transport},
    quic, shadowsocks, udp2tcp,
};

/// Settings for the local socket obfuscator to run: either a single obfuscator or a multiplexer.
#[derive(Debug, Clone)]
pub enum ObfuscationSettings {
    Single(tunnel_obfuscation::Settings),
    Multiplexer {
        transports: Vec<tunnel_obfuscation::multiplexer::Transport>,
        /// Public key of the local WireGuard instance
        client_public_key: PublicKey,
    },
}

impl ObfuscationSettings {
    /// Return the settings of the single obfuscator to run, if this is not a multiplexer.
    pub fn single(&self) -> Option<&tunnel_obfuscation::Settings> {
        match self {
            ObfuscationSettings::Single(settings) => Some(settings),
            ObfuscationSettings::Multiplexer { .. } => None,
        }
    }

    /// The overhead (in bytes) that this obfuscation adds to every packet.
    pub fn packet_overhead(&self) -> u16 {
        match self {
            ObfuscationSettings::Single(settings) => settings.packet_overhead(),
            ObfuscationSettings::Multiplexer { transports, .. } => {
                multiplexer::packet_overhead(transports)
            }
        }
    }
}

pub fn settings_from_config(
    client_public_key: PublicKey,
    server_public_key: PublicKey,
    obfuscation_config: &Obfuscators,
    mtu: u16,
) -> ObfuscationSettings {
    match obfuscation_config {
        Obfuscators::Single(obfuscation_config) => {
            ObfuscationSettings::Single(settings_from_single_config(
                client_public_key,
                server_public_key,
                obfuscation_config,
                mtu,
            ))
        }
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
                );
                transports.push(multiplexer::Transport::Obfuscated(settings));
            }
            ObfuscationSettings::Multiplexer {
                transports,
                client_public_key,
            }
        }
    }
}

fn settings_from_single_config(
    client_public_key: PublicKey,
    server_public_key: PublicKey,
    obfuscation_config: &ObfuscatorConfig,
    mtu: u16,
) -> tunnel_obfuscation::Settings {
    match obfuscation_config {
        ObfuscatorConfig::Udp2Tcp { endpoint } => {
            tunnel_obfuscation::Settings::Udp2Tcp(udp2tcp::Settings { peer: *endpoint })
        }
        ObfuscatorConfig::Shadowsocks { endpoint } => {
            tunnel_obfuscation::Settings::Shadowsocks(shadowsocks::Settings {
                shadowsocks_endpoint: *endpoint,
                wireguard_endpoint: if endpoint.is_ipv4() {
                    SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
                } else {
                    SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
                },
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
            tunnel_obfuscation::Settings::Quic(settings)
        }
        ObfuscatorConfig::Lwo { endpoint, version } => {
            tunnel_obfuscation::Settings::Lwo(lwo::Settings {
                server_addr: *endpoint,
                client_public_key,
                server_public_key,
                version: *version,
            })
        }
    }
}

/// The inverse of [`settings_from_single_config`].
///
/// The settings carry derived state that the config does not -- the local WireGuard endpoint and
/// the MTU -- which is simply dropped.
pub fn config_from_single_settings(settings: &tunnel_obfuscation::Settings) -> ObfuscatorConfig {
    match settings {
        tunnel_obfuscation::Settings::Udp2Tcp(settings) => ObfuscatorConfig::Udp2Tcp {
            endpoint: settings.peer,
        },
        tunnel_obfuscation::Settings::Shadowsocks(settings) => ObfuscatorConfig::Shadowsocks {
            endpoint: settings.shadowsocks_endpoint,
        },
        tunnel_obfuscation::Settings::Quic(settings) => ObfuscatorConfig::Quic {
            hostname: settings.hostname().to_owned(),
            endpoint: settings.quic_endpoint(),
            auth_token: settings.auth_token().to_owned(),
        },
        tunnel_obfuscation::Settings::Lwo(settings) => ObfuscatorConfig::Lwo {
            endpoint: settings.server_addr,
            version: settings.version,
        },
    }
}

/// A running obfuscated transport.
#[derive(Clone)]
pub enum RunningObfuscation {
    /// Rewrite each datagram in place on its way out, over GotaTun's own socket.
    Lwo(lwo::Settings),

    /// Carry the datagrams through this transport, which has a socket of its own.
    Transport(Arc<dyn ObfuscatedTransport>),
}

/// Set up the obfuscation for `settings`.
///
/// The [SelectedTransportRx] is `Some` only for a multiplexer, and returns the selected obfuscation
/// type.
pub async fn create_obfuscation(
    settings: &ObfuscationSettings,
    bypass: Arc<dyn SocketBypass>,
) -> Result<(RunningObfuscation, Option<SelectedTransportRx>)> {
    match settings {
        // LWO is special-cased since `ObfuscatedTransport` does not support batched send/recv.
        ObfuscationSettings::Single(tunnel_obfuscation::Settings::Lwo(settings)) => {
            Ok((RunningObfuscation::Lwo(settings.clone()), None))
        }

        ObfuscationSettings::Single(settings) => {
            let transport = create_transport(bypass, settings)
                .await
                .map_err(Error::ObfuscationError)?;
            Ok((RunningObfuscation::Transport(transport), None))
        }

        ObfuscationSettings::Multiplexer {
            transports,
            client_public_key,
        } => {
            let (selected_transport, selected_transport_rx) = oneshot::channel();
            let multiplexer = Multiplexer::new(
                bypass,
                multiplexer::Settings {
                    transports: transports.clone(),
                    client_public_key: client_public_key.clone(),
                    selected_transport,
                },
            );
            Ok((
                RunningObfuscation::Transport(Arc::new(multiplexer)),
                Some(selected_transport_rx),
            ))
        }
    }
}

/// Told which transport a multiplexer committed to.
pub type SelectedTransportRx = oneshot::Receiver<Transport>;

/// Create the [`SocketBypass`] used to exclude the GotaTun inline obfuscation transport's
/// sockets from tunnel traffic.
pub fn create_socket_bypass(
    #[cfg(target_os = "linux")] config: &crate::config::Config,
    #[cfg(target_os = "android")] tun_provider: Arc<std::sync::Mutex<TunProvider>>,
) -> Arc<dyn SocketBypass> {
    Arc::new(ObfuscatorSocketBypass {
        #[cfg(target_os = "linux")]
        fwmark: config.fwmark.unwrap_or_else(|| {
            log::error!("'fwmark' not set");
            0
        }),

        #[cfg(target_os = "android")]
        tun_provider,
    })
}

pub struct ObfuscatorSocketBypass {
    #[cfg(target_os = "linux")]
    pub fwmark: u32,

    #[cfg(target_os = "android")]
    pub tun_provider: Arc<std::sync::Mutex<TunProvider>>,
}

impl SocketBypass for ObfuscatorSocketBypass {
    #[cfg(target_os = "linux")]
    fn bypass_socket(
        &self,
        socket: socket2::SockRef<'_>,
        _token: &BypassToken,
    ) -> std::io::Result<()> {
        socket.set_mark(self.fwmark)
    }

    #[cfg(any(windows, target_os = "macos"))]
    fn bypass_socket(
        &self,
        _socket: socket2::SockRef<'_>,
        _token: &BypassToken,
    ) -> std::io::Result<()> {
        Ok(())
    }

    #[cfg(target_os = "android")]
    fn bypass_socket(
        &self,
        socket: socket2::SockRef<'_>,
        _token: &BypassToken,
    ) -> std::io::Result<()> {
        use std::os::unix::io::AsRawFd;

        self.tun_provider
            .lock()
            .unwrap()
            .bypass(&socket.as_raw_fd())
            .map_err(std::io::Error::other)
    }

    fn revoke_bypass(
        &self,
        _socket: socket2::SockRef<'_>,
        _token: &BypassToken,
    ) -> std::io::Result<()> {
        Ok(())
    }
}
