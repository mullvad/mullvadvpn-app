//! Glue between tunnel-obfuscation and WireGuard configurations

use super::{Error, Result};
use crate::{CloseMsg, config::Config};
#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex};
use std::{net::SocketAddr, sync::mpsc as sync_mpsc};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::TunProvider;
use talpid_types::ErrorExt;
use tunnel_obfuscation::create_local_socket_obfuscator;

/// Begin running obfuscation machine, if configured. This function will patch `config`'s endpoint
/// to point to an endpoint on localhost.
///
/// # Arguments
///
/// * close_msg_sender - channel to send close messages on failure
/// * tun_provider - (Android only) used to bypass the VPN for the remote socket
pub async fn run_local_socket_obfuscator(
    config: &mut Config,
    close_msg_sender: sync_mpsc::Sender<CloseMsg>,
    #[cfg(target_os = "android")] tun_provider: Arc<Mutex<TunProvider>>,
) -> Result<Option<ObfuscatorHandle>> {
    let Some(ref obfuscation_settings) = config.obfuscation_settings else {
        return Ok(None);
    };
    let obfuscator = create_local_socket_obfuscator(obfuscation_settings)
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

/// Returns `true` when the obfuscation config can be applied inline in userspace WireGuard
/// (GotaTun), avoiding the need for a local socket obfuscator.
pub fn userspace_transport_available(
    params: &talpid_types::net::wireguard::TunnelParameters,
) -> bool {
    use talpid_types::net::obfuscation::{ObfuscatorConfig, Obfuscators};
    matches!(
        params.obfuscation.as_ref(),
        Some(Obfuscators::Single(ObfuscatorConfig::Lwo { .. }))
            | Some(Obfuscators::Single(ObfuscatorConfig::Quic { .. }))
    )
}

/// Patch the first peer in the WireGuard configuration to use the local proxy endpoint
fn patch_endpoint(config: &mut Config, endpoint: SocketAddr) {
    log::trace!("Patching first WireGuard peer to become {endpoint}");
    config.entry_peer.endpoint = endpoint;
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
