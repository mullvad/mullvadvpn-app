//! Negotiate ephemeral peers before the tunnel is opened.

use crate::{
    CloseMsg, Error,
    config::Config,
    gotatun::{MaybeObfuscatingTransportFactory, lwo_timer_params, lwo_version},
    obfuscation::RunningObfuscation,
};
use std::{io, net::Ipv4Addr, sync::Arc, time::Duration};
use talpid_net::bypass::SocketBypass;
use talpid_tunnel_config_client::negotiation::{
    self, Ingress, IngressTransport, NegotiatedPeers, NegotiationConfig, NegotiationError, Relay,
    Relays,
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
    let negotiation_config = NegotiationConfig {
        private_key: config.tunnel.private_key.clone(),
        tunnel_ipv4: config.tunnel_ipv4().unwrap_or(Ipv4Addr::UNSPECIFIED),
        tunnel_ipv6: config.tunnel_ipv6(),
        config_service_ip: config.ipv4_gateway,
        relays: relays(config),
        enable_post_quantum: config.quantum_resistant,
        enable_daita: config.daita,
        timeout,
    };
    let mut transport = GotaTunIngressTransport {
        config,
        obfuscation,
        bypass,
    };

    negotiation::negotiate_ephemeral_peers(&negotiation_config, &mut transport)
        .await
        .map_err(|error| match error {
            NegotiationError::Timeout => {
                log::warn!(
                    "Timeout while negotiating ephemeral peers \
                     (retry {retry_attempt}, timeout {timeout:?}, PQ={}, DAITA={})",
                    negotiation_config.enable_post_quantum,
                    negotiation_config.enable_daita,
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

/// Reaches the ingress relay the same way as the tunnel does.
struct GotaTunIngressTransport<'a> {
    config: &'a Config,
    obfuscation: Option<&'a RunningObfuscation>,
    bypass: &'a Arc<dyn SocketBypass>,
}

impl IngressTransport for GotaTunIngressTransport<'_> {
    type Factory = MaybeObfuscatingTransportFactory;
    type Guard = ();

    async fn connect(
        &mut self,
        client_public_key: &PublicKey,
    ) -> io::Result<Ingress<Self::Factory, Self::Guard>> {
        let endpoint = self.config.entry_peer.endpoint;
        let obfuscation = self
            .obfuscation
            .cloned()
            .map(|obfuscation| obfuscation.with_client_public_key(client_public_key.clone()));
        // The temporary devices carry little traffic, so the socket buffers are left as they are.
        let factory = MaybeObfuscatingTransportFactory::new(
            false,
            obfuscation,
            endpoint,
            Arc::clone(self.bypass),
        );
        let timer_params =
            (lwo_version(self.config) == Some(LwoVersion::V2)).then(lwo_timer_params);

        Ok(Ingress {
            factory,
            endpoint,
            timer_params,
            guard: (),
        })
    }
}
