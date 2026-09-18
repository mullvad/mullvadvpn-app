use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use gotatun::{
    udp::{UdpTransportFactory, UdpTransportFactoryParams, socket::UdpSocket},
    x25519::StaticSecret,
};
use talpid_net::bypass::NoopBypass;
use tokio::sync::Mutex;
use tunnel_obfuscation::{
    create_transport,
    gotatun_transport::{
        MaybeObfuscatingRecv, MaybeObfuscatingSend, MaybeObfuscatingTransportFactory,
        RunningObfuscation,
    },
};

use super::BoundUdpTransports;
use super::params::{ObfuscationParameters, TunnelParameters};

/// The ingress device's UDP transport: the pre-bound socket, obfuscated in place.
///
/// The obfuscation is held the way [`BoundUdpTransports`] holds the socket beneath it: a device
/// reads either only when it binds, so both are replaced by suspending the device, swapping, and
/// waking it.
#[derive(Clone)]
pub struct ObfuscatingTransports {
    udp: BoundUdpTransports,
    obfuscation: Arc<Mutex<Option<RunningObfuscation>>>,
    /// The relay this addresses. See [`MaybeObfuscatingTransportFactory`].
    peer_endpoint: SocketAddr,
}

impl ObfuscatingTransports {
    pub fn new(
        udp: BoundUdpTransports,
        obfuscation: Option<RunningObfuscation>,
        peer_endpoint: SocketAddr,
    ) -> Self {
        Self {
            udp,
            obfuscation: Arc::new(Mutex::new(obfuscation)),
            peer_endpoint,
        }
    }

    /// Replace the obfuscation. The device uses it from its next bind on.
    pub async fn replace(&self, obfuscation: Option<RunningObfuscation>) {
        *self.obfuscation.lock().await = obfuscation;
    }
}

impl UdpTransportFactory for ObfuscatingTransports {
    type Send = MaybeObfuscatingSend<UdpSocket>;
    type Recv = MaybeObfuscatingRecv<UdpSocket>;

    async fn bind(
        &mut self,
        params: &UdpTransportFactoryParams,
    ) -> io::Result<(Self::Send, Self::Recv)> {
        let obfuscation = self.obfuscation.lock().await.clone();
        MaybeObfuscatingTransportFactory::new(self.udp.clone(), obfuscation, self.peer_endpoint)
            .bind(params)
            .await
    }
}

pub enum ObfuscationProxyError {
    InvalidQuicToken(String),
    LocalSocketError(tunnel_obfuscation::Error),
}

impl std::fmt::Display for ObfuscationProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObfuscationProxyError::InvalidQuicToken(msg) => write!(f, "Invalid QUIC token: {msg}"),
            ObfuscationProxyError::LocalSocketError(msg) => write!(f, "Local socket error: {msg}"),
        }
    }
}

/// Create the obfuscation that reaches the ingress relay. It is used by the temporary devices
/// that negotiate ephemeral peers, and then reused by the tunnel devices.
/// Returns `None` if obfuscation is off.
pub async fn create_obfuscation(
    params: &TunnelParameters,
) -> Result<Option<RunningObfuscation>, ObfuscationProxyError> {
    let obfuscation = match obfuscation_settings(params)? {
        None => return Ok(None),
        // LWO obfuscates each datagram in place, over the socket of the device.
        Some(tunnel_obfuscation::Settings::Lwo(settings)) => RunningObfuscation::Lwo(settings),
        Some(settings) => RunningObfuscation::Transport(
            create_transport(Arc::new(NoopBypass), &settings)
                .await
                .map_err(ObfuscationProxyError::LocalSocketError)?,
        ),
    };
    Ok(Some(obfuscation))
}

/// The settings of the obfuscator that reaches the ingress relay.
/// Returns `None` if obfuscation is off.
fn obfuscation_settings(
    params: &TunnelParameters,
) -> Result<Option<tunnel_obfuscation::Settings>, ObfuscationProxyError> {
    let ingress_endpoint = params.ingress_peer().endpoint;

    let settings = match &params.obfuscation {
        ObfuscationParameters::Off => return Ok(None),
        ObfuscationParameters::UdpOverTcp => {
            tunnel_obfuscation::Settings::Udp2Tcp(tunnel_obfuscation::udp2tcp::Settings {
                peer: ingress_endpoint,
            })
        }
        ObfuscationParameters::Shadowsocks => {
            let wg_ep = localhost_wg_endpoint(ingress_endpoint);
            tunnel_obfuscation::Settings::Shadowsocks(tunnel_obfuscation::shadowsocks::Settings {
                shadowsocks_endpoint: ingress_endpoint,
                wireguard_endpoint: wg_ep,
            })
        }
        ObfuscationParameters::Quic { hostname, token } => {
            let wg_ep = localhost_wg_endpoint(ingress_endpoint);
            let token = token
                .parse::<tunnel_obfuscation::quic::AuthToken>()
                .map_err(ObfuscationProxyError::InvalidQuicToken)?;
            tunnel_obfuscation::Settings::Quic(tunnel_obfuscation::quic::Settings::new(
                ingress_endpoint,
                hostname.clone(),
                token,
                wg_ep,
            ))
        }
        ObfuscationParameters::Lwo { server_public_key } => {
            // Placeholder client key: every user of these settings overrides it with the key
            // of the device the obfuscation is for, via `with_client_public_key`.
            let device_public_key =
                gotatun::x25519::PublicKey::from(&StaticSecret::from(params.private_key));
            tunnel_obfuscation::Settings::Lwo(tunnel_obfuscation::lwo::Settings {
                server_addr: ingress_endpoint,
                client_public_key: talpid_types::net::wireguard::PublicKey::from(
                    device_public_key.to_bytes(),
                ),
                server_public_key: talpid_types::net::wireguard::PublicKey::from(
                    *server_public_key,
                ),
                version: talpid_types::net::obfuscation::LwoVersion::V1,
            })
        }
    };
    Ok(Some(settings))
}

fn localhost_wg_endpoint(peer: SocketAddr) -> SocketAddr {
    if peer.is_ipv4() {
        SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
    } else {
        SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localhost_wg_endpoint_matches_family() {
        assert_eq!(
            localhost_wg_endpoint("1.2.3.4:51820".parse().unwrap()),
            SocketAddr::from((Ipv4Addr::LOCALHOST, 51820))
        );
        assert_eq!(
            localhost_wg_endpoint("[2001:db8::1]:51820".parse().unwrap()),
            SocketAddr::from((Ipv6Addr::LOCALHOST, 51820))
        );
    }
}
