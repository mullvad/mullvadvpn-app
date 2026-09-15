//! Glue between tunnel-obfuscation and WireGuard configurations

use super::{Error, Result};
use crate::CloseMsg;
#[cfg(target_os = "android")]
use std::sync::Mutex;
use std::{
    iter,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{Arc, mpsc as sync_mpsc},
};
use talpid_net::bypass::{BypassToken, SocketBypass};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::TunProvider;
use talpid_types::{
    ErrorExt,
    net::{
        obfuscation::{ObfuscatorConfig, Obfuscators},
        wireguard::{PeerConfig, PublicKey},
    },
};
use tokio::sync::oneshot;
use tunnel_obfuscation::{
    LocalSocketObfuscator, create_local_socket_obfuscator_with_bypass, lwo,
    multiplexer::{self, Transport},
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

    /// See [`tunnel_obfuscation::Settings::stacks_on_provided_socket`].
    pub fn stacks_on_provided_socket(&self) -> bool {
        match self {
            ObfuscationSettings::Single(settings) => settings.stacks_on_provided_socket(),
            // Each transport it races has a socket of its own.
            ObfuscationSettings::Multiplexer(_) => false,
        }
    }
}

/// Begin running obfuscation machine, if configured. This function will patch `config`'s endpoint
/// to point to an endpoint on localhost.
///
/// # Arguments
///
/// * close_msg_sender - channel to send close messages on failure
/// * bypass - socket bypass for excluding obfuscator sockets from the tunnel
pub async fn spawn_local_socket_obfuscator(
    entry_peer: &mut PeerConfig,
    obfuscation_settings: ObfuscationSettings,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
    bypass: Arc<dyn SocketBypass>,
) -> Result<ObfuscatorHandle> {
    log::trace!("Obfuscation settings: {obfuscation_settings:?}");

    let mut selected_transport_rx = None;

    let obfuscator: Box<dyn LocalSocketObfuscator> = match obfuscation_settings {
        ObfuscationSettings::Single(settings) => {
            create_local_socket_obfuscator_with_bypass(bypass, &settings)
                .await
                .map_err(Error::ObfuscationError)?
        }
        ObfuscationSettings::Multiplexer {
            transports,
            client_public_key,
        } => {
            let (selected_transport_tx, selected_transport) = oneshot::channel();
            let settings = multiplexer::Settings {
                transports,
                client_public_key,
                selected_transport: selected_transport_tx,
            };
            // The multiplexer connects to one out of several endpoints, and only commits to one
            // of them. Have it announce its choice so that the firewall can be
            // tightened accordingly.
            selected_transport_rx = Some(selected_transport);
            Box::new(
                multiplexer::Multiplexer::new(bypass, settings)
                    .await
                    .map_err(Error::ObfuscationError)?,
            )
        }
    };

    patch_endpoint(entry_peer, obfuscator.endpoint());

    let obfuscation_task = tokio::spawn(async move {
        match obfuscator.run().await {
            Ok(_) => {
                let _ = close_msg_sender.send(CloseMsg::ObfuscatorExpired);
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Obfuscation controller failed")
                );
                let _ = close_msg_sender
                    .send(CloseMsg::ObfuscatorFailed(Error::ObfuscationError(error)));
            }
        }
    });

    Ok(ObfuscatorHandle {
        obfuscation_task,
        selected_transport_rx,
    })
}

/// Returns `true` when the obfuscation config can be applied inline in userspace WireGuard
/// (GotaTun), avoiding the need for a local socket obfuscator.
///
/// Every single obfuscation method is an `ObfuscatedTransport`, which GotaTun drives directly.
/// The multiplexer is not: it races several transports against each other and only commits to
/// one of them once it answers, so it is still reached over a local socket.
pub fn userspace_transport_available(
    params: &talpid_types::net::wireguard::TunnelParameters,
) -> bool {
    matches!(params.obfuscation.as_ref(), Some(Obfuscators::Single(_)))
}

/// Patch the first peer in the WireGuard configuration to use the local proxy endpoint
fn patch_endpoint(entry_peer: &mut PeerConfig, endpoint: SocketAddr) {
    log::trace!("Patching first WireGuard peer to become {endpoint}");
    entry_peer.endpoint = endpoint;
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

/// Simple wrapper that automatically cancels the future which runs an obfuscator.
pub struct ObfuscatorHandle {
    obfuscation_task: tokio::task::JoinHandle<()>,
    selected_transport_rx: Option<oneshot::Receiver<Transport>>,
}

impl ObfuscatorHandle {
    pub fn abort(&self) {
        self.obfuscation_task.abort();
    }

    /// Notified with the transport that the obfuscator commits to.
    ///
    /// Only a multiplexer has a choice to make, so this is `None` for every other obfuscator.
    pub fn take_selected_transport_rx(&mut self) -> Option<oneshot::Receiver<Transport>> {
        self.selected_transport_rx.take()
    }
}

impl Drop for ObfuscatorHandle {
    fn drop(&mut self) {
        self.obfuscation_task.abort();
    }
}

/// Create the [`SocketBypass`] used for both the local socket obfuscator and the GotaTun
/// inline obfuscation transport.
pub fn create_socket_bypass(
    #[cfg(target_os = "linux")] config: &crate::config::Config,
    #[cfg(target_os = "android")] tun_provider: Arc<Mutex<TunProvider>>,
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
    pub tun_provider: Arc<Mutex<TunProvider>>,
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
