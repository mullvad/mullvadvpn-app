//! Glue between talpid-obfuscation and WireGuard configurations

use super::{Error, Result};
use crate::{config::Config, CloseMsg};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::mpsc as sync_mpsc,
};
use talpid_types::net::obfuscation::ObfuscatorConfig;
use talpid_types::ErrorExt;

use tunnel_obfuscation::{
    create_obfuscator, shadowsocks, udp2tcp, Settings as ObfuscationSettings,
};

/// Begin running obfuscation machine, if configured. This function will patch `config`'s endpoint
/// to point to an endpoint on localhost
pub async fn apply_obfuscation_config(
    config: &mut Config,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
) -> Result<Option<ObfuscatorHandle>> {
    if let Some(ref obfuscator_config) = config.obfuscator_config {
        let settings = settings_from_config(
            obfuscator_config,
            #[cfg(target_os = "linux")]
            config.fwmark,
        );
        apply_obfuscation_config_inner(config, settings, close_msg_sender)
            .await
            .map(Some)
    } else {
        Ok(None)
    }
}

async fn apply_obfuscation_config_inner(
    config: &mut Config,
    settings: ObfuscationSettings,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
) -> Result<ObfuscatorHandle> {
    log::trace!("Obfuscation settings: {settings:?}");

    let obfuscator = create_obfuscator(&settings)
        .await
        .map_err(Error::ObfuscationError)?;

    #[cfg(target_os = "android")]
    let remote_socket_fd = obfuscator.remote_socket_fd();

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

    Ok(ObfuscatorHandle::new(
        obfuscation_task,
        #[cfg(target_os = "android")]
        remote_socket_fd,
    ))
}

/// Patch the first peer in the WireGuard configuration to use the local proxy endpoint
fn patch_endpoint(config: &mut Config, endpoint: SocketAddr) {
    log::trace!("Patching first WireGuard peer to become {endpoint}");
    config.entry_peer.endpoint = endpoint;
}

fn settings_from_config(
    config: &ObfuscatorConfig,
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
    }
}

/// Simple wrapper that automatically cancels the future which runs an obfuscator.
pub struct ObfuscatorHandle {
    obfuscation_task: tokio::task::JoinHandle<()>,
    #[cfg(target_os = "android")]
    remote_socket_fd: std::os::unix::io::RawFd,
}

impl ObfuscatorHandle {
    pub fn new(
        obfuscation_task: tokio::task::JoinHandle<()>,
        #[cfg(target_os = "android")] remote_socket_fd: std::os::unix::io::RawFd,
    ) -> Self {
        Self {
            obfuscation_task,
            #[cfg(target_os = "android")]
            remote_socket_fd,
        }
    }

    #[cfg(target_os = "android")]
    pub fn remote_socket_fd(&self) -> std::os::unix::io::RawFd {
        self.remote_socket_fd
    }

    pub fn abort(&self) {
        self.obfuscation_task.abort();
    }
}

impl Drop for ObfuscatorHandle {
    fn drop(&mut self) {
        self.obfuscation_task.abort();
    }
}
