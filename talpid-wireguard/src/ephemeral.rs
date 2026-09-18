//! Negotiate ephemeral peers before the tunnel is opened.

use crate::{
    CloseMsg, Error,
    config::Config,
    connectivity,
    gotatun::{lwo_timer_params, lwo_version, transport_factory},
    obfuscation::RunningObfuscation,
};
use std::{sync::Arc, time::Duration};
use talpid_net::bypass::SocketBypass;
use talpid_tunnel_config_client::negotiation::{
    self, Negotiables, NegotiatedPeers, NegotiationConfig, NegotiationError, Relay, Relays,
};
use talpid_types::net::{
    obfuscation::LwoVersion,
    wireguard::{PeerConfig, PublicKey},
};

const INITIAL_PSK_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_PSK_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(48);
const PSK_EXCHANGE_TIMEOUT_MULTIPLIER: u32 = 2;

/// Negotiate ephemeral peers with the relays in `config`.
///
/// The relays are reached through temporary GotaTun devices, whichever WireGuard implementation the
/// tunnel uses, so the tunnel does not have to be open.
pub async fn negotiate_ephemeral_peers(
    config: &Config,
    retry_attempt: u32,
    obfuscation: Option<&RunningObfuscation>,
    bypass: &Arc<dyn SocketBypass>,
) -> Result<NegotiatedPeers, CloseMsg> {
    let timeout = psk_exchange_timeout(retry_attempt);
    // An unreachable relay should fail as fast as the connectivity check would.
    let handshake_timeout = connectivity::establish_timeout(retry_attempt);
    let negotiate = Negotiables {
        post_quantum: config.quantum_resistant,
        daita: config.daita,
    };
    let negotiation_config = NegotiationConfig {
        private_key: config.tunnel.private_key.clone(),
        tunnel_ipv4: config.tunnel_ipv4().ok_or(CloseMsg::SetupError(
            Error::WireguardConfigError(crate::config::Error::InvalidTunnelIpError),
        ))?,
        config_service_ip: config.ipv4_gateway,
        relays: relays(config),
        ingress_timer_params: (lwo_version(config) == Some(LwoVersion::V2)).then(lwo_timer_params),
        timeout,
        handshake_timeout,
        tcp_timeout: Some(handshake_timeout),
        // `Config` has a single private key, since some tunnels use one device for multihop.
        separate_exit_key: false,
    };
    // Reach the ingress relay the same way as the tunnel does. The temporary devices carry little
    // traffic, so the socket buffers are left as they are.
    let ingress_transport = |client_public_key: &PublicKey| {
        let obfuscation = obfuscation
            .cloned()
            .map(|obfuscation| obfuscation.with_client_public_key(client_public_key.clone()));
        transport_factory(
            false,
            obfuscation,
            config.entry_peer.endpoint,
            Arc::clone(bypass),
        )
    };

    negotiation::negotiate_ephemeral_peers(&negotiation_config, negotiate, ingress_transport)
        .await
        .map_err(|error| match error {
            NegotiationError::Timeout => {
                log::warn!(
                    "Timeout while negotiating ephemeral peers \
                     (retry {retry_attempt}, timeout {timeout:?}, \
                     handshake timeout {handshake_timeout:?}, PQ={}, DAITA={})",
                    negotiate.post_quantum,
                    negotiate.daita,
                );
                CloseMsg::EphemeralPeerNegotiationTimeout
            }
            error => CloseMsg::SetupError(Error::EphemeralPeerNegotiationError(error)),
        })
}

/// How long to wait for a config service to respond, on attempt `retry_attempt` to connect.
fn psk_exchange_timeout(retry_attempt: u32) -> Duration {
    std::cmp::min(
        MAX_PSK_EXCHANGE_TIMEOUT,
        INITIAL_PSK_EXCHANGE_TIMEOUT
            .saturating_mul(PSK_EXCHANGE_TIMEOUT_MULTIPLIER.saturating_pow(retry_attempt)),
    )
}

/// The relays of `config` to negotiate ephemeral peers with.
fn relays(config: &Config) -> Relays {
    let relay = |peer: &PeerConfig| Relay {
        public_key: peer.public_key.clone(),
        endpoint: peer.endpoint,
    };
    match &config.exit_peer {
        None => Relays::Singlehop(relay(&config.entry_peer)),
        Some(exit_peer) => Relays::Multihop {
            entry: relay(&config.entry_peer),
            exit: relay(exit_peer),
        },
    }
}
