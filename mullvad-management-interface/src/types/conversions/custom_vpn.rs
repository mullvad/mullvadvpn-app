use crate::types::{
    FromProtobufTypeError,
    conversions::{arg_from_str, bytes_to_privkey, bytes_to_pubkey},
    proto,
};
use talpid_types::net::wireguard::{
    CustomVpnConfig, CustomVpnConfigUpdate, CustomVpnPeerConfig, CustomVpnSettings,
    CustomVpnTunnelUpdate,
};
use vec1::Vec1;

fn parse_tunnel_ips(ips: &[String]) -> Result<Vec<std::net::IpAddr>, FromProtobufTypeError> {
    if ips.is_empty() {
        return Err(FromProtobufTypeError::InvalidArgument(
            "tunnel requires at least one IP".to_owned(),
        ));
    }
    ips.iter()
        .map(|ip| arg_from_str(ip, "invalid tunnel IP address"))
        .collect()
}

fn parse_peers(
    peers: Vec<proto::custom_vpn_config::PeerConfig>,
) -> Result<Vec1<CustomVpnPeerConfig>, FromProtobufTypeError> {
    let peers = peers
        .into_iter()
        .map(|p| {
            Ok(CustomVpnPeerConfig {
                public_key: bytes_to_pubkey(&p.public_key)?,
                allowed_ip: p
                    .allowed_ip
                    .iter()
                    .map(|ip| arg_from_str(ip, "invalid allowed IP"))
                    .collect::<Result<_, _>>()?,
                endpoint: arg_from_str(&p.endpoint, "invalid endpoint")?,
            })
        })
        .collect::<Result<Vec<_>, FromProtobufTypeError>>()?;
    Vec1::try_from_vec(peers)
        .map_err(|_| FromProtobufTypeError::InvalidArgument("missing peer config".to_owned()))
}

/// Parse a config exchanged with a client into a
/// [`CustomVpnConfigUpdate`]. An omitted private key keeps the stored
/// one; see [`CustomVpnConfigUpdate::resolve`].
impl TryFrom<proto::CustomVpnConfig> for CustomVpnConfigUpdate {
    type Error = FromProtobufTypeError;

    fn try_from(config: proto::CustomVpnConfig) -> Result<Self, Self::Error> {
        let tunnel = config
            .tunnel
            .map(|t| {
                Ok::<_, FromProtobufTypeError>(CustomVpnTunnelUpdate {
                    private_key: t
                        .private_key
                        .map(|key| bytes_to_privkey(&key))
                        .transpose()?,
                    ips: parse_tunnel_ips(&t.ips)?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing tunnel config".to_owned(),
            ))??;

        Ok(CustomVpnConfigUpdate {
            enabled: config.enabled,
            name: config.name,
            tunnel,
            peers: parse_peers(config.peers)?,
        })
    }
}

impl TryFrom<proto::CustomVpnSettings> for Vec<CustomVpnConfigUpdate> {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::CustomVpnSettings) -> Result<Self, Self::Error> {
        settings
            .configs
            .into_iter()
            .map(CustomVpnConfigUpdate::try_from)
            .collect()
    }
}

/// Read path: build a proto response from the stored settings. On Android
/// the daemon and the app run with the same privileges, so the private
/// key is included there. Any local user may reach the desktop RPC
/// interface, so the key is withheld on desktop.
// TODO: on desktop, include the key only when the caller is sufficiently
// privileged, e.g. running as root.
impl From<&CustomVpnSettings> for proto::CustomVpnSettings {
    fn from(settings: &CustomVpnSettings) -> Self {
        proto::CustomVpnSettings {
            configs: settings.configs.iter().map(config_to_proto).collect(),
        }
    }
}

fn config_to_proto(config: &CustomVpnConfig) -> proto::CustomVpnConfig {
    proto::CustomVpnConfig {
        tunnel: Some(proto::custom_vpn_config::TunnelConfig {
            private_key: response_private_key(config),
            ips: config.tunnel.ips.iter().map(ToString::to_string).collect(),
        }),
        peers: config
            .peers
            .iter()
            .map(|peer| proto::custom_vpn_config::PeerConfig {
                public_key: peer.public_key.as_bytes().to_vec(),
                allowed_ip: peer.allowed_ip.iter().map(|ip| ip.to_string()).collect(),
                endpoint: peer.endpoint.to_string(),
            })
            .collect(),
        enabled: config.enabled,
        name: config.name.clone(),
    }
}

#[cfg(target_os = "android")]
fn response_private_key(config: &CustomVpnConfig) -> Option<Vec<u8>> {
    Some(config.tunnel.private_key.to_bytes().to_vec())
}

#[cfg(not(target_os = "android"))]
fn response_private_key(_config: &CustomVpnConfig) -> Option<Vec<u8>> {
    None
}

/// Write path: build the proto request from an update.
impl From<CustomVpnConfigUpdate> for proto::CustomVpnConfig {
    fn from(update: CustomVpnConfigUpdate) -> Self {
        proto::CustomVpnConfig {
            tunnel: Some(proto::custom_vpn_config::TunnelConfig {
                private_key: update.tunnel.private_key.map(|key| key.to_bytes().to_vec()),
                ips: update.tunnel.ips.iter().map(ToString::to_string).collect(),
            }),
            peers: update
                .peers
                .iter()
                .map(|peer| proto::custom_vpn_config::PeerConfig {
                    public_key: peer.public_key.as_bytes().to_vec(),
                    allowed_ip: peer.allowed_ip.iter().map(|ip| ip.to_string()).collect(),
                    endpoint: peer.endpoint.to_string(),
                })
                .collect(),
            enabled: update.enabled,
            name: update.name,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use talpid_types::net::wireguard::{CustomVpnConfig, CustomVpnTunnelConfig, PrivateKey};

    const KEY_BYTES: [u8; 32] = [7; 32];

    fn sample_config() -> CustomVpnConfig {
        CustomVpnConfig {
            enabled: true,
            name: Some("test".to_string()),
            tunnel: CustomVpnTunnelConfig {
                private_key: PrivateKey::from(KEY_BYTES),
                ips: vec!["10.0.0.2".parse().unwrap(), "fd00::2".parse().unwrap()],
            },
            peers: vec1::vec1![CustomVpnPeerConfig {
                public_key: bytes_to_pubkey(&[1; 32]).unwrap(),
                allowed_ip: vec!["0.0.0.0/0".parse().unwrap()],
                endpoint: "1.2.3.4:51820".parse().unwrap(),
            }],
        }
    }

    fn sample_proto_config(private_key: Option<Vec<u8>>) -> proto::CustomVpnConfig {
        proto::CustomVpnConfig {
            tunnel: Some(proto::custom_vpn_config::TunnelConfig {
                private_key,
                ips: vec!["10.0.0.2".to_string()],
            }),
            peers: vec![proto::custom_vpn_config::PeerConfig {
                public_key: vec![1; 32],
                allowed_ip: vec!["0.0.0.0/0".to_string()],
                endpoint: "1.2.3.4:51820".to_string(),
            }],
            enabled: true,
            name: None,
        }
    }

    #[test]
    fn test_response_private_key_policy() {
        let settings = CustomVpnSettings {
            configs: vec![sample_config()],
        };

        let proto_settings = proto::CustomVpnSettings::from(&settings);
        assert_eq!(proto_settings.configs.len(), 1);
        let proto_config = proto_settings.configs.first().unwrap();
        let tunnel = proto_config.tunnel.as_ref().unwrap();
        if cfg!(target_os = "android") {
            assert_eq!(tunnel.private_key, Some(KEY_BYTES.to_vec()));
        } else {
            assert!(tunnel.private_key.is_none());
        }
        // The rest of the config survives the conversion
        assert_eq!(
            tunnel.ips,
            vec!["10.0.0.2".to_string(), "fd00::2".to_string()]
        );
        assert!(proto_config.enabled);
        assert_eq!(proto_config.name.as_deref(), Some("test"));
        assert_eq!(proto_config.peers.len(), 1);
    }

    #[test]
    fn test_settings_round_trip() {
        let settings = CustomVpnSettings {
            configs: vec![sample_config()],
        };

        let updates =
            Vec::<CustomVpnConfigUpdate>::try_from(proto::CustomVpnSettings::from(&settings))
                .expect("valid settings");
        assert_eq!(updates.len(), 1);

        // On Android the key is included in responses; elsewhere it is
        // withheld and an absent key keeps the stored one.
        if cfg!(target_os = "android") {
            assert_eq!(
                updates[0].tunnel.private_key,
                Some(PrivateKey::from(KEY_BYTES))
            );
        } else {
            assert!(updates[0].tunnel.private_key.is_none());
        }

        // Resolving the read-back update against the stored config
        // reconstructs it on both platforms.
        let resolved = updates
            .into_iter()
            .next()
            .unwrap()
            .resolve(settings.configs.first())
            .expect("resolvable");
        assert_eq!(resolved, settings.configs[0]);
    }

    #[test]
    fn test_update_from_proto_with_and_without_key() {
        let absent =
            CustomVpnConfigUpdate::try_from(sample_proto_config(None)).expect("valid config");
        assert!(absent.tunnel.private_key.is_none());

        let present =
            CustomVpnConfigUpdate::try_from(sample_proto_config(Some(KEY_BYTES.to_vec())))
                .expect("valid config");
        assert_eq!(
            present.tunnel.private_key,
            Some(PrivateKey::from(KEY_BYTES))
        );

        // A present key that is not a valid WireGuard key is rejected
        let invalid = CustomVpnConfigUpdate::try_from(sample_proto_config(Some(vec![1; 31])));
        assert!(invalid.is_err());
    }

    #[test]
    fn test_resolve_key_handling() {
        let existing = sample_config();

        // Absent key + existing config -> keep the existing key
        let absent =
            CustomVpnConfigUpdate::try_from(sample_proto_config(None)).expect("valid config");
        let resolved = absent.resolve(Some(&existing)).expect("resolvable");
        assert_eq!(resolved.tunnel.private_key, existing.tunnel.private_key);

        // Absent key + no existing config -> error
        let absent =
            CustomVpnConfigUpdate::try_from(sample_proto_config(None)).expect("valid config");
        assert!(absent.resolve(None).is_err());

        // Present key -> used even if there is an existing config
        let new_key = PrivateKey::from([9; 32]);
        let present = CustomVpnConfigUpdate::try_from(sample_proto_config(Some([9; 32].to_vec())))
            .expect("valid config");
        let resolved = present.resolve(Some(&existing)).expect("resolvable");
        assert_eq!(resolved.tunnel.private_key, new_key);
    }
}
