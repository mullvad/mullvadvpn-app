//! Post-quantum / DAITA negotiation with the relays' in-tunnel config services.

use std::{
    io,
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
};

use gotatun::{
    device::{DeviceBuilder, Peer},
    udp::channel::new_udp_tun_channel,
    x25519::{PublicKey, StaticSecret},
};
use ipnetwork::IpNetwork;
use talpid_tunnel_config_client::{EphemeralPeer, RelayConfigService, request_ephemeral_peer_with};
use talpid_types::net::wireguard::PrivateKey;
use tonic::transport::channel::Endpoint;
use tower::util::service_fn;

use crate::{
    gotatun::smoltcp_network::{SmoltcpHandle, smoltcp_network},
    tunnel_adapter::NegotiatePQError,
};

use super::{
    BoundUdpTransports,
    obfuscation::{ObfuscationProxy, ObfuscationSlot},
    params::{PeerParameters, TunnelParameters},
};

const CONFIG_SERVICE_ADDR: &str = "10.64.0.1:1337";

/// Key material one hop's device is configured with.
#[derive(Clone)]
pub struct HopKeys {
    pub private_key: StaticSecret,
    pub preshared_key: Option<[u8; 32]>,
}

impl HopKeys {
    /// The device key and no preshared key: what a hop uses before, or without, negotiation.
    pub fn device(params: &TunnelParameters) -> Self {
        Self {
            private_key: params.device_key(),
            preshared_key: None,
        }
    }

    fn ephemeral(private_key: PrivateKey, negotiated: &EphemeralPeer) -> Self {
        Self {
            private_key: StaticSecret::from(private_key.to_bytes()),
            preshared_key: negotiated.psk.as_ref().map(|psk| *psk.as_bytes()),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self.private_key)
    }
}

/// Keys negotiated with the relays. `entry` is present exactly when the tunnel is multihop.
///
/// Without PQ/DAITA nothing is negotiated and every hop uses the device key.
pub struct NegotiatedKeys {
    pub entry: Option<HopKeys>,
    pub exit: HopKeys,
}

impl NegotiatedKeys {
    fn unnegotiated(params: &TunnelParameters) -> Self {
        Self {
            entry: params.is_multihop().then(|| HopKeys::device(params)),
            exit: HopKeys::device(params),
        }
    }

    /// Keys of the device talking to the ingress relay.
    pub fn ingress(&self) -> &HopKeys {
        self.entry.as_ref().unwrap_or(&self.exit)
    }
}

/// Negotiate the post-quantum / DAITA ephemeral peer(s), or return the device keys when
/// neither is enabled.
pub async fn negotiate(
    params: &TunnelParameters,
    udp: &BoundUdpTransports,
    obfuscation: &mut ObfuscationSlot,
    stopped: &AtomicBool,
) -> Result<NegotiatedKeys, NegotiatePQError> {
    if !(params.enable_pq || params.enable_daita) {
        return Ok(NegotiatedKeys::unnegotiated(params));
    }

    let ingress_keys = negotiate_with_ingress(params, udp, obfuscation).await?;

    let Some(entry) = params.entry_peer.as_ref() else {
        return Ok(NegotiatedKeys {
            entry: None,
            exit: ingress_keys,
        });
    };
    if stopped.load(Ordering::SeqCst) {
        return Err(NegotiatePQError::Timeout);
    }

    let exit_keys =
        negotiate_with_exit_via_entry(params, udp, obfuscation, entry, &ingress_keys).await?;
    Ok(NegotiatedKeys {
        entry: Some(ingress_keys),
        exit: exit_keys,
    })
}

/// Phase 1: negotiate with the ingress relay (the entry in multihop, the exit otherwise)
/// using the device key.
async fn negotiate_with_ingress(
    params: &TunnelParameters,
    udp: &BoundUdpTransports,
    obfuscation: &mut ObfuscationSlot,
) -> Result<HopKeys, NegotiatePQError> {
    let ingress = params.ingress_peer();
    let device_keys = HopKeys::device(params);
    log::info!(
        "PQ phase 1: negotiating with {} (pq={}, daita={})",
        ingress.endpoint,
        params.enable_pq,
        params.enable_daita
    );

    let proxy = obfuscation
        .for_key(params, device_keys.public_key())
        .await
        .map_err(NegotiatePQError::ObfuscationProxyError)?;
    let endpoint = ingress_endpoint(ingress, proxy);

    let (handle, ip_recv, ip_send, _guard) = smoltcp_network(params.smoltcp_network_config());
    let device = DeviceBuilder::new()
        .with_udp(udp.clone())
        .with_ip_pair(ip_send, ip_recv)
        .with_private_key(device_keys.private_key.clone())
        .with_peer(config_service_peer(ingress.public_key, endpoint))
        .build()
        .await
        .map_err(NegotiatePQError::DeviceError)?;

    let ephemeral_private = PrivateKey::new_from_random();
    let result = tokio::time::timeout(
        params.establish_timeout(),
        negotiate_ephemeral_peer(
            &handle,
            params,
            ephemeral_private.public_key(),
            params.enable_daita,
        ),
    )
    .await;

    device.stop().await;

    let ephemeral = finish_exchange(
        NegotiatePQError::ExchangeError,
        NegotiatePQError::Timeout,
        "PQ phase 1",
        result,
    )?;
    Ok(HopKeys::ephemeral(ephemeral_private, &ephemeral))
}

/// Phase 2 (multihop only): reach the exit relay's config service through the entry, whose
/// device now uses the phase-1 ephemeral key, so that the config service address hits the
/// exit relay. The exit hop itself still uses the device key here.
async fn negotiate_with_exit_via_entry(
    params: &TunnelParameters,
    udp: &BoundUdpTransports,
    obfuscation: &mut ObfuscationSlot,
    entry: &PeerParameters,
    entry_keys: &HopKeys,
) -> Result<HopKeys, NegotiatePQError> {
    log::info!("PQ phase 2: negotiating with exit via entry ephemeral key");

    let proxy = obfuscation
        .for_key(params, entry_keys.public_key())
        .await
        .map_err(NegotiatePQError::Phase2ObfuscationError)?;
    let entry_endpoint = ingress_endpoint(entry, proxy);

    let (handle, ip_recv, ip_send, _guard) = smoltcp_network(params.smoltcp_network_config());
    let (ch_tx, ch_rx, udp_ch) = new_udp_tun_channel(
        100,
        params.ipv4_addr,
        params.ipv6_addr,
        params.entry_mtu(entry),
    );

    let exit_device = DeviceBuilder::new()
        .with_udp(udp_ch)
        .with_ip_pair(ip_send, ip_recv)
        .with_peer(config_service_peer(
            params.exit_peer.public_key,
            params.exit_peer.endpoint,
        ))
        .with_private_key(params.device_key())
        .build()
        .await
        .map_err(NegotiatePQError::Phase2ExitDeviceError)?;

    let mut entry_peer = Peer::new(entry.public_key.into())
        .with_endpoint(entry_endpoint)
        .with_allowed_ip(IpNetwork::from(params.exit_peer.endpoint.ip()));
    if let Some(psk) = entry_keys.preshared_key {
        entry_peer = entry_peer.with_preshared_key(psk);
    }
    let entry_device = match DeviceBuilder::new()
        .with_udp(udp.clone())
        .with_ip_pair(ch_tx, ch_rx)
        .with_peer(entry_peer)
        .with_private_key(entry_keys.private_key.clone())
        .build()
        .await
    {
        Ok(dev) => dev,
        Err(e) => {
            exit_device.stop().await;
            return Err(NegotiatePQError::Phase2EntryDeviceError(e));
        }
    };

    let ephemeral_private = PrivateKey::new_from_random();
    let result = tokio::time::timeout(
        params.establish_timeout(),
        negotiate_ephemeral_peer(&handle, params, ephemeral_private.public_key(), false),
    )
    .await;

    entry_device.stop().await;
    exit_device.stop().await;

    let ephemeral = finish_exchange(
        NegotiatePQError::Phase2ExchangeError,
        NegotiatePQError::Phase2Timeout,
        "PQ phase 2",
        result,
    )?;
    Ok(HopKeys::ephemeral(ephemeral_private, &ephemeral))
}

fn finish_exchange<F>(
    phase_error: F,
    timeout_error: NegotiatePQError,
    phase: &str,
    result: Result<Result<EphemeralPeer, String>, tokio::time::error::Elapsed>,
) -> Result<EphemeralPeer, NegotiatePQError>
where
    F: Fn(String) -> NegotiatePQError,
{
    match result {
        Ok(Ok(ephemeral)) => {
            log::info!(
                "{phase} complete (psk={}, daita={})",
                ephemeral.psk.is_some(),
                ephemeral.daita.is_some()
            );
            Ok(ephemeral)
        }
        // Phase 1 or phase2 error ?
        Ok(Err(e)) => Err(phase_error(e)),
        Err(_) => Err(timeout_error),
    }
}

/// Endpoint to use to reach an ingress peer.
pub fn ingress_endpoint(
    ingress: &PeerParameters,
    obfuscation: Option<&ObfuscationProxy>,
) -> SocketAddr {
    obfuscation
        .map(|proxy| proxy.endpoint())
        .unwrap_or(ingress.endpoint)
}

/// A peer restricted to the relay's in-tunnel config service ([`CONFIG_SERVICE_ADDR`]), so
/// negotiation cannot reach the wider internet.
fn config_service_peer(public_key: [u8; 32], endpoint: SocketAddr) -> Peer {
    Peer::new(public_key.into())
        .with_endpoint(endpoint)
        .with_allowed_ip(IpNetwork::from(config_service_addr().ip()))
}

fn config_service_addr() -> SocketAddr {
    CONFIG_SERVICE_ADDR
        .parse()
        .expect("CONFIG_SERVICE_ADDR is a valid socket address")
}

/// Negotiate an ephemeral peer via gRPC through the smoltcp TCP stack.
async fn negotiate_ephemeral_peer(
    smoltcp_handle: &SmoltcpHandle,
    params: &TunnelParameters,
    ephemeral_pubkey: talpid_types::net::wireguard::PublicKey,
    enable_daita: bool,
) -> Result<EphemeralPeer, String> {
    let stream = smoltcp_handle
        .tcp_connect(config_service_addr())
        .await
        .map_err(|e| format!("TCP connect to config service: {e}"))?;

    // The connector is called exactly once by tonic; use a Mutex to hand off the stream.
    let mut stream_cell = Some(stream);

    let endpoint = Endpoint::from_static("tcp://0.0.0.0:0");
    let conn = endpoint
        .connect_with_connector(service_fn(move |_| {
            let stream = stream_cell
                .take()
                .ok_or_else(|| io::Error::other("connector stream unavailable"));
            async move { Ok::<_, io::Error>(hyper_util::rt::tokio::TokioIo::new(stream?)) }
        }))
        .await
        .map_err(|e| format!("gRPC connect: {e}"))?;

    let parent_pubkey = PrivateKey::from(params.private_key).public_key();
    request_ephemeral_peer_with(
        RelayConfigService::new(conn),
        parent_pubkey,
        ephemeral_pubkey,
        params.enable_pq,
        enable_daita,
    )
    .await
    .map_err(|e| format!("Ephemeral peer exchange: {e}"))
}

#[cfg(test)]
mod tests {
    use super::super::params::tests::{params, peer};
    use super::*;

    #[test]
    fn unnegotiated_keys_follow_hop_layout() {
        let single = NegotiatedKeys::unnegotiated(&params());
        assert!(single.entry.is_none());
        assert_eq!(
            single.exit.private_key.to_bytes(),
            params().device_key().to_bytes()
        );

        let mut p = params();
        p.entry_peer = Some(peer("9.9.9.9:51820"));
        let multi = NegotiatedKeys::unnegotiated(&p);
        assert!(multi.entry.is_some());
    }

    #[test]
    fn ingress_keys_are_entry_when_multihop_else_exit() {
        let entry = HopKeys {
            private_key: StaticSecret::from([1u8; 32]),
            preshared_key: None,
        };
        let exit = HopKeys {
            private_key: StaticSecret::from([2u8; 32]),
            preshared_key: None,
        };

        let multihop = NegotiatedKeys {
            entry: Some(entry.clone()),
            exit: exit.clone(),
        };
        assert_eq!(multihop.ingress().public_key(), entry.public_key());

        let singlehop = NegotiatedKeys { entry: None, exit };
        assert_eq!(
            singlehop.ingress().public_key(),
            PublicKey::from(&StaticSecret::from([2u8; 32]))
        );
    }

    #[test]
    fn config_service_peer_is_restricted_to_config_service() {
        let p = config_service_peer([7u8; 32], "1.2.3.4:51820".parse().unwrap());
        assert_eq!(
            p.allowed_ips,
            vec![IpNetwork::from(config_service_addr().ip())]
        );
        assert_eq!(p.endpoint, Some("1.2.3.4:51820".parse().unwrap()));
    }

    #[test]
    fn ingress_endpoint_falls_back_to_relay_without_proxy() {
        let ingress = peer("1.2.3.4:51820");
        assert_eq!(ingress_endpoint(&ingress, None), ingress.endpoint);
    }
}
