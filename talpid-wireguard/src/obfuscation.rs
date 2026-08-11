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
        Endpoint,
        obfuscation::{ObfuscatorConfig, Obfuscators},
        wireguard::{PeerConfig, PublicKey},
    },
};
use tokio::sync::watch;
use tunnel_obfuscation::{
    Settings as ObfuscationSettings, create_local_socket_obfuscator_with_bypass, lwo, multiplexer,
    quic, shadowsocks, udp2tcp,
};

/// Begin running obfuscation machine, if configured. This function will patch `config`'s endpoint
/// to point to an endpoint on localhost.
///
/// # Arguments
///
/// * close_msg_sender - channel to send close messages on failure
/// * tun_provider - (Android only) used to bypass the VPN for the remote socket
/// * fwmark - (Linux only) firewall mark to apply to the obfuscator's remote socket
pub async fn spawn_local_socket_obfuscator(
    entry_peer: &mut PeerConfig,
    mut obfuscation_settings: tunnel_obfuscation::Settings,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
    #[cfg(target_os = "android")] tun_provider: Arc<Mutex<TunProvider>>,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Result<ObfuscatorHandle> {
    log::trace!("Obfuscation settings: {obfuscation_settings:?}");

    // The multiplexer connects to one out of several endpoints, and only commits to one of them
    // at runtime. Have it announce its choice so that the firewall can be tightened accordingly.
    let (selected_endpoint_tx, selected_endpoint) =
        watch::channel(obfuscation_settings.remote_endpoint());
    if let ObfuscationSettings::Multiplexer(settings) = &mut obfuscation_settings {
        settings.selected_endpoint = Some(Arc::new(selected_endpoint_tx));
    }

    let bypass = Arc::new(ObfuscatorSocketBypass {
        #[cfg(target_os = "linux")]
        fwmark: fwmark.unwrap_or_else(|| {
            log::error!("'fwmark' not set");
            0
        }),

        #[cfg(target_os = "android")]
        tun_provider,
    });

    let obfuscator = create_local_socket_obfuscator_with_bypass(bypass, &obfuscation_settings)
        .await
        .map_err(Error::ObfuscationError)?;

    let packet_overhead = obfuscator.packet_overhead();

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
        packet_overhead,
        selected_endpoint,
    })
}

/// Returns `true` when the obfuscation config can be applied inline in userspace WireGuard
/// (GotaTun), avoiding the need for a local socket obfuscator.
pub fn userspace_transport_available(
    params: &talpid_types::net::wireguard::TunnelParameters,
) -> bool {
    matches!(
        params.obfuscation.as_ref(),
        Some(Obfuscators::Single(ObfuscatorConfig::Lwo { .. }))
            | Some(Obfuscators::Single(ObfuscatorConfig::Quic { .. }))
    )
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
        Obfuscators::Single(obfuscation_config) => settings_from_single_config(
            client_public_key,
            server_public_key,
            obfuscation_config,
            mtu,
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
                );
                transports.push(multiplexer::Transport::Obfuscated(settings));
            }
            ObfuscationSettings::Multiplexer(multiplexer::Settings {
                transports,
                selected_endpoint: None,
            })
        }
    }
}

fn settings_from_single_config(
    client_public_key: PublicKey,
    server_public_key: PublicKey,
    obfuscation_config: &ObfuscatorConfig,
    mtu: u16,
) -> ObfuscationSettings {
    match obfuscation_config {
        ObfuscatorConfig::Udp2Tcp { endpoint } => {
            ObfuscationSettings::Udp2Tcp(udp2tcp::Settings { peer: *endpoint })
        }
        ObfuscatorConfig::Shadowsocks { endpoint } => {
            ObfuscationSettings::Shadowsocks(shadowsocks::Settings {
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
            ObfuscationSettings::Quic(settings)
        }
        ObfuscatorConfig::Lwo { endpoint } => ObfuscationSettings::Lwo(lwo::Settings {
            server_addr: *endpoint,
            client_public_key,
            server_public_key,
        }),
    }
}

/// Simple wrapper that automatically cancels the future which runs an obfuscator.
pub struct ObfuscatorHandle {
    obfuscation_task: tokio::task::JoinHandle<()>,
    packet_overhead: u16,
    selected_endpoint: watch::Receiver<Option<Endpoint>>,
}

impl ObfuscatorHandle {
    pub fn abort(&self) {
        self.obfuscation_task.abort();
    }

    pub fn packet_overhead(&self) -> u16 {
        self.packet_overhead
    }

    /// The remote endpoint that the obfuscator connects to, if it has committed to one.
    ///
    /// This is known up front for all obfuscators except the multiplexer, which only decides
    /// once one of its transports has responded.
    pub fn selected_endpoint(&self) -> Option<Endpoint> {
        *self.selected_endpoint.borrow()
    }
}

impl Drop for ObfuscatorHandle {
    fn drop(&mut self) {
        self.obfuscation_task.abort();
    }
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
