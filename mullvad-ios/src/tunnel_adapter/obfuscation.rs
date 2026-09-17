use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

use gotatun::x25519::PublicKey;
use talpid_types::net::{obfuscation::LwoVersion, wireguard};
use tunnel_obfuscation::{Settings, create_local_socket_obfuscator};

use crate::tunnel_adapter::ObfuscationProxyError;

use super::params::{ObfuscationParameters, TunnelParameters};

/// A running obfuscation proxy. Aborted on drop.
pub struct ObfuscationProxy {
    endpoint: SocketAddr,
    /// Client key the transport is bound to. Only set for LWO.
    client_public_key: Option<PublicKey>,
    task: tokio::task::JoinHandle<()>,
}

impl ObfuscationProxy {
    /// `None` when obfuscation is off.
    async fn start(
        params: &TunnelParameters,
        client_public_key: PublicKey,
    ) -> Result<Option<Self>, ObfuscationProxyError> {
        let Some(settings) = settings(params, client_public_key)? else {
            return Ok(None);
        };

        let obfuscator = create_local_socket_obfuscator(&settings)
            .await
            .map_err(ObfuscationProxyError::LocalSocketError)?;
        let endpoint = obfuscator.endpoint();
        log::info!(
            "Obfuscation proxy towards {} started at {endpoint}",
            params.ingress_peer().endpoint
        );
        let task = tokio::spawn(async move {
            let _ = obfuscator.run().await;
        });
        Ok(Some(Self {
            endpoint,
            client_public_key: matches!(settings, Settings::Lwo(_)).then_some(client_public_key),
            task,
        }))
    }

    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    fn serves(&self, client_public_key: PublicKey) -> bool {
        self.client_public_key
            .is_none_or(|bound| bound == client_public_key)
    }
}

impl Drop for ObfuscationProxy {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Holds one obfuscation proxy, replacing it only when a device key it cannot serve is
/// requested.
#[derive(Default)]
pub struct ObfuscationSlot {
    proxy: Option<ObfuscationProxy>,
}

impl ObfuscationSlot {
    /// `None` when obfuscation is off.
    pub async fn for_key(
        &mut self,
        params: &TunnelParameters,
        client_public_key: PublicKey,
    ) -> Result<Option<&ObfuscationProxy>, ObfuscationProxyError> {
        let reusable = self
            .proxy
            .as_ref()
            .is_some_and(|proxy| proxy.serves(client_public_key));
        if !reusable {
            self.proxy = ObfuscationProxy::start(params, client_public_key).await?;
        }
        Ok(self.proxy.as_ref())
    }

    pub fn reset(&mut self) {
        self.proxy = None;
    }
}

/// Obfuscator settings for the ingress relay, or `None` when obfuscation is off.
fn settings(
    params: &TunnelParameters,
    client_public_key: PublicKey,
) -> Result<Option<Settings>, ObfuscationProxyError> {
    let ingress = params.ingress_peer().endpoint;
    let settings = match &params.obfuscation {
        ObfuscationParameters::Off => return Ok(None),
        ObfuscationParameters::UdpOverTcp => {
            Settings::Udp2Tcp(tunnel_obfuscation::udp2tcp::Settings { peer: ingress })
        }
        ObfuscationParameters::Shadowsocks => {
            Settings::Shadowsocks(tunnel_obfuscation::shadowsocks::Settings {
                shadowsocks_endpoint: ingress,
                wireguard_endpoint: localhost_wg_endpoint(ingress),
            })
        }
        ObfuscationParameters::Quic { hostname, token } => {
            let token = token
                .parse::<tunnel_obfuscation::quic::AuthToken>()
                .map_err(ObfuscationProxyError::InvalidQuicToken)?;
            Settings::Quic(tunnel_obfuscation::quic::Settings::new(
                ingress,
                hostname.clone(),
                token,
                localhost_wg_endpoint(ingress),
            ))
        }
        ObfuscationParameters::Lwo { server_public_key } => {
            Settings::Lwo(tunnel_obfuscation::lwo::Settings {
                server_addr: ingress,
                client_public_key: wireguard::PublicKey::from(client_public_key.to_bytes()),
                server_public_key: wireguard::PublicKey::from(*server_public_key),
                version: LwoVersion::V1,
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
    use super::super::params::tests::{params, peer};
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

    #[tokio::test]
    async fn key_bound_proxy_serves_only_its_key() {
        let proxy = |key| ObfuscationProxy {
            endpoint: "127.0.0.1:1".parse().unwrap(),
            client_public_key: key,
            task: tokio::spawn(async {}),
        };
        let a = PublicKey::from([1u8; 32]);
        let b = PublicKey::from([2u8; 32]);

        assert!(proxy(None).serves(a));
        assert!(proxy(None).serves(b));
        assert!(proxy(Some(a)).serves(a));
        assert!(!proxy(Some(a)).serves(b));
    }

    #[tokio::test]
    async fn slot_off_yields_no_proxy_and_stays_empty() {
        let mut slot = ObfuscationSlot::default();
        let key = PublicKey::from([1u8; 32]);
        assert!(slot.for_key(&params(), key).await.unwrap().is_none());
        assert!(slot.proxy.is_none());
    }

    #[test]
    fn off_yields_no_settings() {
        let key = PublicKey::from([1u8; 32]);
        assert!(settings(&params(), key).unwrap().is_none());
    }

    #[test]
    fn lwo_targets_ingress_relay_with_given_client_key() {
        let mut p = params();
        p.entry_peer = Some(peer("9.9.9.9:51820"));
        p.obfuscation = ObfuscationParameters::Lwo {
            server_public_key: [9u8; 32],
        };
        let client = PublicKey::from([1u8; 32]);

        let Some(Settings::Lwo(lwo)) = settings(&p, client).unwrap() else {
            panic!("expected LWO settings");
        };
        assert_eq!(lwo.server_addr, "9.9.9.9:51820".parse().unwrap());
        assert_eq!(lwo.client_public_key.as_bytes(), &[1u8; 32]);
        assert_eq!(lwo.server_public_key.as_bytes(), &[9u8; 32]);
    }
}
