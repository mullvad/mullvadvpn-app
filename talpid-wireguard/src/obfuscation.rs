//! Glue between tunnel-obfuscation and WireGuard configurations

use super::{Error, Result};
use crate::{CloseMsg, config::Config};
#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::mpsc as sync_mpsc,
};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::TunProvider;
use talpid_types::{ErrorExt, net::obfuscation::ObfuscatorConfig};

use tunnel_obfuscation::{
    Obfuscator, Settings as ObfuscationSettings, create_obfuscator,
    multiplexer::{self, Multiplexer},
    quic, shadowsocks, udp2tcp,
};

/// Begin running obfuscation machine, if configured. This function will patch `config`'s endpoint
/// to point to an endpoint on localhost
///
/// # Arguments
///
/// * obfuscation_mtu - "MTU" including obfuscation overhead
pub async fn apply_obfuscation_config(
    config: &mut Config,
    obfuscation_mtu: u16,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
    #[cfg(target_os = "android")] tun_provider: Arc<Mutex<TunProvider>>,
) -> Result<Option<ObfuscatorHandle>> {
    let Some(ref obfuscator_config) = config.obfuscator_config else {
        log::info!("** Running obfuscation multiplexer **");
        let entry_addr = config.entry_peer.endpoint;

        let direct = multiplexer::Transport::Direct(entry_addr);

        let udp2tcp =
            multiplexer::Transport::Obfuscated(ObfuscationSettings::Udp2Tcp(udp2tcp::Settings {
                peer: SocketAddr::new(entry_addr.ip(), 443),
                #[cfg(target_os = "linux")]
                fwmark: config.fwmark,
            }));
        let shadowsocks = multiplexer::Transport::Obfuscated(ObfuscationSettings::Shadowsocks(
            shadowsocks::Settings {
                shadowsocks_endpoint: SocketAddr::new(entry_addr.ip(), 51900),
                wireguard_endpoint: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 51820),
                #[cfg(target_os = "linux")]
                fwmark: config.fwmark,
            },
        ));

        let multiplexer_settings = multiplexer::Settings {
            transports: vec![udp2tcp, shadowsocks, direct],
        };

        let Ok(obfuscator) = Multiplexer::new(multiplexer_settings).await else {
            return Ok(None);
        };

        let packet_overhead = obfuscator.packet_overhead();

        patch_endpoint(config, obfuscator.endpoint());

        let obfuscation_task = tokio::spawn(async move {
            obfuscator.start().await.unwrap();
            let _ = close_msg_sender.send(CloseMsg::ObfuscatorExpired);
        });

        return Ok(Some(ObfuscatorHandle {
            obfuscation_task,
            packet_overhead,
        }));
    };

    let settings = settings_from_config(
        obfuscator_config,
        obfuscation_mtu,
        #[cfg(target_os = "linux")]
        config.fwmark,
    );

    log::trace!("Obfuscation settings: {settings:?}");
    let obfuscator = create_obfuscator(&settings)
        .await
        .map_err(Error::ObfuscationError)?;

    let packet_overhead = obfuscator.packet_overhead();

    #[cfg(target_os = "android")]
    bypass_vpn(tun_provider, obfuscator.remote_socket_fd()).await;

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

/// Patch the first peer in the WireGuard configuration to use the local proxy endpoint
fn patch_endpoint(config: &mut Config, endpoint: SocketAddr) {
    log::trace!("Patching first WireGuard peer to become {endpoint}");
    config.entry_peer.endpoint = endpoint;
}

fn settings_from_config(
    config: &ObfuscatorConfig,
    mtu: u16,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> ObfuscationSettings {
    match config {
        ObfuscatorConfig::Udp2Tcp { endpoint } => ObfuscationSettings::Udp2Tcp(udp2tcp::Settings {
            peer: *endpoint,
            #[cfg(target_os = "linux")]
            fwmark,
        }),
        ObfuscatorConfig::Shadowsocks { endpoint } => {
            ObfuscationSettings::Shadowsocks(shadowsocks::Settings {
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
                return ObfuscationSettings::Quic(settings.fwmark(fwmark));
            }
            ObfuscationSettings::Quic(settings)
        }
    }
}

/// Route socket outside of the VPN on Android
#[cfg(target_os = "android")]
async fn bypass_vpn(
    tun_provider: Arc<Mutex<TunProvider>>,
    remote_socket_fd: std::os::unix::io::RawFd,
) {
    // Exclude remote obfuscation socket or bridge
    log::debug!("Excluding remote socket fd from the tunnel");
    let _ = tokio::task::spawn_blocking(move || {
        if let Err(error) = tun_provider.lock().unwrap().bypass(&remote_socket_fd) {
            log::error!("Failed to exclude remote socket fd: {error}");
        }
    })
    .await;
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
