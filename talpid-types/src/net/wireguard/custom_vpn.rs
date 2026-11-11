//! Types for the user-provided custom WireGuard VPN.
use std::net::{IpAddr, SocketAddr};

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use vec1::Vec1;

use super::{PrivateKey, PublicKey};
/// Custom VPN tunnel interface configuration (WireGuard-based).
///
/// `ips` are the interface addresses, matching wg-quick `Address = ...`
/// lines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomVpnTunnelConfig {
    pub private_key: PrivateKey,
    pub ips: Vec<IpAddr>,
}

/// Custom VPN peer configuration (WireGuard-based).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomVpnPeerConfig {
    pub public_key: PublicKey,
    pub allowed_ip: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
}

/// Custom VPN configuration (WireGuard-based).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomVpnConfig {
    pub enabled: bool,
    pub name: Option<String>,
    pub tunnel: CustomVpnTunnelConfig,
    pub peers: Vec1<CustomVpnPeerConfig>,
}

impl CustomVpnConfig {
    /// The peer used by the tunnel. Only a single peer is currently supported.
    pub fn peer(&self) -> &CustomVpnPeerConfig {
        self.peers.first()
    }
}

/// Custom VPN configs (WireGuard-based).
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomVpnSettings {
    pub configs: Vec<CustomVpnConfig>,
}

impl CustomVpnSettings {
    /// The config to use, which is the first enabled config, if any.
    pub fn active(&self) -> Option<&CustomVpnConfig> {
        self.configs.iter().find(|config| config.enabled)
    }
}

/// A custom VPN config as exchanged with a client. The private key is
/// optional: when submitted, an omitted key keeps the stored one; see
/// [`CustomVpnConfigUpdate::resolve`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomVpnConfigUpdate {
    pub enabled: bool,
    pub name: Option<String>,
    pub tunnel: CustomVpnTunnelUpdate,
    pub peers: Vec1<CustomVpnPeerConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomVpnTunnelUpdate {
    pub private_key: Option<PrivateKey>,
    pub ips: Vec<IpAddr>,
}

impl From<CustomVpnConfig> for CustomVpnConfigUpdate {
    fn from(config: CustomVpnConfig) -> Self {
        CustomVpnConfigUpdate {
            enabled: config.enabled,
            name: config.name,
            tunnel: CustomVpnTunnelUpdate {
                private_key: Some(config.tunnel.private_key),
                ips: config.tunnel.ips,
            },
            peers: config.peers,
        }
    }
}

impl CustomVpnConfigUpdate {
    /// Resolve the update into a full config. The update's key is used when
    /// present, otherwise the key of `existing`. Errors if there is neither.
    pub fn resolve(self, existing: Option<&CustomVpnConfig>) -> Result<CustomVpnConfig, String> {
        let private_key = match self.tunnel.private_key {
            Some(key) => key,
            None => existing
                .map(|config| config.tunnel.private_key.clone())
                .ok_or_else(|| {
                    "cannot set config without a private key: no existing key to keep".to_owned()
                })?,
        };

        Ok(CustomVpnConfig {
            enabled: self.enabled,
            name: self.name,
            tunnel: CustomVpnTunnelConfig {
                private_key,
                ips: self.tunnel.ips,
            },
            peers: self.peers,
        })
    }
}
