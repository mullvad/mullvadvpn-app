//! Reach the relays before the tunnel is opened, through temporary GotaTun devices, to negotiate
//! ephemeral peers.

use crate::{
    CloseMsg, Error,
    config::Config,
    connectivity,
    gotatun::{TransportFactory, lwo_timer_params, lwo_version, transport_factory},
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

/// A WireGuard session with the ingress relay.
pub type IngressSession = negotiation::IngressSession<TransportFactory>;

/// Open a session with the ingress relay of `config`, through `obfuscation`.
///
/// The session uses a temporary GotaTun device, whichever WireGuard implementation the tunnel uses,
/// so it does not depend on a tunnel device.
pub async fn open_ingress_session(
    config: &Config,
    retry_attempt: u32,
    obfuscation: Option<RunningObfuscation>,
    bypass: &Arc<dyn SocketBypass>,
) -> Result<IngressSession, CloseMsg> {
    let negotiation_config = negotiation_config(config, retry_attempt)?;
    let transport = ingress_transport(
        config,
        obfuscation.as_ref(),
        bypass,
        &config.tunnel.private_key.public_key(),
    );
    IngressSession::new(&negotiation_config, transport)
        .await
        .map_err(|error| CloseMsg::SetupError(Error::EphemeralPeerNegotiationError(error)))
}

/// Complete a handshake with the ingress relay through `session`.
pub async fn handshake(
    session: &IngressSession,
    config: &Config,
    retry_attempt: u32,
) -> Result<(), CloseMsg> {
    let negotiation_config = negotiation_config(config, retry_attempt)?;
    session
        .handshake(&negotiation_config)
        .await
        .map_err(|error| match error {
            NegotiationError::Timeout => {
                log::warn!(
                    "Timeout while completing a handshake with the ingress relay (retry {retry_attempt}, \
                     handshake timeout {:?})",
                    negotiation_config.handshake_timeout,
                );
                CloseMsg::PingErr
            }
            error => CloseMsg::SetupError(Error::EphemeralPeerNegotiationError(error)),
        })
}

/// Negotiate ephemeral peers with the relays in `config`, starting with the ingress relay, which
/// is reached through `ingress`.
///
/// In multihop, the exit relay is reached through the entry relay, by temporary GotaTun devices
/// that reach the entry relay through `obfuscation`.
pub async fn negotiate_ephemeral_peers(
    config: &Config,
    retry_attempt: u32,
    ingress: IngressSession,
    obfuscation: Option<&RunningObfuscation>,
    bypass: &Arc<dyn SocketBypass>,
) -> Result<NegotiatedPeers, CloseMsg> {
    let negotiation_config = negotiation_config(config, retry_attempt)?;
    let negotiate = Negotiables {
        post_quantum: config.quantum_resistant,
        daita: config.daita,
    };
    let entry_transport = |client_public_key: &PublicKey| {
        ingress_transport(config, obfuscation, bypass, client_public_key)
    };

    negotiation::negotiate_ephemeral_peers_over(
        &negotiation_config,
        negotiate,
        ingress,
        entry_transport,
    )
    .await
    .map_err(|error| match error {
        NegotiationError::Timeout => {
            log::warn!(
                "Timeout while negotiating ephemeral peers \
                 (retry {retry_attempt}, timeout {:?}, \
                 handshake timeout {:?}, PQ={}, DAITA={})",
                negotiation_config.timeout,
                negotiation_config.handshake_timeout,
                negotiate.post_quantum,
                negotiate.daita,
            );
            CloseMsg::EphemeralPeerNegotiationTimeout
        }
        error => CloseMsg::SetupError(Error::EphemeralPeerNegotiationError(error)),
    })
}

/// The transport of a temporary device with `client_public_key`, which reaches the ingress relay
/// the same way as the tunnel does.
fn ingress_transport(
    config: &Config,
    obfuscation: Option<&RunningObfuscation>,
    bypass: &Arc<dyn SocketBypass>,
    client_public_key: &PublicKey,
) -> TransportFactory {
    let obfuscation = obfuscation
        .cloned()
        .map(|obfuscation| obfuscation.with_client_public_key(client_public_key.clone()));
    transport_factory(
        false,
        obfuscation,
        config.entry_peer.endpoint,
        Arc::clone(bypass),
    )
}

/// Parameters for negotiating with the relays of `config`, on attempt `retry_attempt` to connect.
fn negotiation_config(config: &Config, retry_attempt: u32) -> Result<NegotiationConfig, CloseMsg> {
    // An unreachable relay should fail as fast as the connectivity check would.
    let handshake_timeout = connectivity::establish_timeout(retry_attempt);
    Ok(NegotiationConfig {
        private_key: config.tunnel.private_key.clone(),
        tunnel_ipv4: config.tunnel_ipv4().ok_or(CloseMsg::SetupError(
            Error::WireguardConfigError(crate::config::Error::InvalidTunnelIpError),
        ))?,
        config_service_ip: config.ipv4_gateway,
        relays: relays(config),
        ingress_timer_params: (lwo_version(config) == Some(LwoVersion::V2)).then(lwo_timer_params),
        timeout: psk_exchange_timeout(retry_attempt),
        handshake_timeout,
        tcp_timeout: Some(handshake_timeout),
        // `Config` has a single private key, since some tunnels use one device for multihop.
        separate_exit_key: false,
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
