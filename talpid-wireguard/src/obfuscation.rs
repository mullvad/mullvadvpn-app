//! Glue between tunnel-obfuscation and WireGuard configurations

use super::{Error, Result};
use crate::{CloseMsg, config::Config};
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
    net::obfuscation::{ObfuscatorConfig, Obfuscators},
};

use tunnel_obfuscation::{
    Settings as ObfuscationSettings, create_obfuscator_with_bypass, lwo, multiplexer, quic,
    shadowsocks, udp2tcp,
};

/// Begin running obfuscation machine, if configured. This function will patch `config`'s endpoint
/// to point to an endpoint on localhost.
///
/// # Arguments
///
/// * obfuscation_mtu - "MTU" including obfuscation overhead
/// * is_gotatun - `true` when the userspace GotaTun implementation is in use
pub async fn apply_obfuscation_config(
    config: &mut Config,
    obfuscation_mtu: u16,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
    is_gotatun: bool,
    #[cfg(target_os = "android")] tun_provider: Arc<Mutex<TunProvider>>,
) -> Result<Option<ObfuscatorHandle>> {
    let Some(ref obfuscator_config) = config.obfuscator_config else {
        return Ok(None);
    };

    // When GotaTun is in use and LWO is configured, obfuscation is applied inline by
    // MaybeObfuscatingTransportFactory.
    if is_gotatun && is_single_lwo(obfuscator_config) {
        log::debug!("GotaTun + LWO: skipping proxy, obfuscation will be applied inline");
        return Ok(None);
    }

    let settings = settings_from_config(config, obfuscator_config, obfuscation_mtu);

    log::trace!("Obfuscation settings: {settings:?}");

    let bypass = Arc::new(ObfuscatorSocketBypass {
        #[cfg(target_os = "linux")]
        fwmark: config.fwmark.unwrap_or_else(|| {
            log::error!("'fwmark' not set");
            0
        }),

        #[cfg(target_os = "android")]
        tun_provider,
    });

    let obfuscator = create_obfuscator_with_bypass(bypass, &settings)
        .await
        .map_err(Error::ObfuscationError)?;

    let packet_overhead = obfuscator.packet_overhead();

    patch_endpoint(config, obfuscator.endpoint());

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

    Ok(Some(ObfuscatorHandle {
        obfuscation_task,
        packet_overhead,
    }))
}

/// Returns `true` when the obfuscation config is a single LWO method.
pub fn is_single_lwo(obfuscators: &Obfuscators) -> bool {
    matches!(
        obfuscators,
        Obfuscators::Single(ObfuscatorConfig::Lwo { .. })
    )
}

/// Patch the first peer in the WireGuard configuration to use the local proxy endpoint
fn patch_endpoint(config: &mut Config, endpoint: SocketAddr) {
    log::trace!("Patching first WireGuard peer to become {endpoint}");
    config.entry_peer.endpoint = endpoint;
}

fn settings_from_config(
    config: &Config,
    obfuscation_config: &Obfuscators,
    mtu: u16,
) -> ObfuscationSettings {
    match obfuscation_config {
        Obfuscators::Single(obfuscation_config) => {
            settings_from_single_config(config, obfuscation_config, mtu)
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
                let settings = settings_from_single_config(config, obfs_config, mtu);
                transports.push(multiplexer::Transport::Obfuscated(settings));
            }
            ObfuscationSettings::Multiplexer(multiplexer::Settings { transports })
        }
    }
}

fn settings_from_single_config(
    config: &Config,
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
            client_public_key: config.tunnel.private_key.public_key(),
            server_public_key: config.entry_peer.public_key.clone(),
        }),
    }
}

/// Simple wrapper that automatically cancels the future which runs an obfuscator.
pub struct ObfuscatorHandle {
    obfuscation_task: tokio::task::JoinHandle<()>,
    packet_overhead: u16,
}

impl ObfuscatorHandle {
    pub fn abort(&self) {
        self.obfuscation_task.abort();
    }

    pub fn packet_overhead(&self) -> u16 {
        self.packet_overhead
    }
}

impl Drop for ObfuscatorHandle {
    fn drop(&mut self) {
        self.obfuscation_task.abort();
    }
}

struct ObfuscatorSocketBypass {
    #[cfg(target_os = "linux")]
    fwmark: u32,

    #[cfg(target_os = "android")]
    tun_provider: Arc<Mutex<TunProvider>>,
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
        // TODO
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
