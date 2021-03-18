use chrono::{offset::Utc, DateTime};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::fmt;
use talpid_types::net::wireguard;

/// Contains account specific wireguard data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WireguardData {
    pub private_key: wireguard::PrivateKey,
    pub addresses: AssociatedAddresses,
    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,
}

impl WireguardData {
    /// Create a public key
    pub fn get_public_key(&self) -> PublicKey {
        PublicKey {
            key: self.private_key.public_key(),
            created: self.created,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(
    target_os = "android",
    jnix(class_name = "net.mullvad.mullvadvpn.model.WireguardTunnelOptions")
)]
pub struct TunnelOptions {
    #[serde(flatten)]
    pub options: wireguard::TunnelOptions,
    /// Interval used for automatic key rotation, in hours
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub automatic_rotation: Option<u32>,
}

/// Represents a published public key
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct PublicKey {
    #[cfg_attr(target_os = "android", jnix(map = "|key| *key.as_bytes()"))]
    pub key: wireguard::PublicKey,
    #[cfg_attr(target_os = "android", jnix(map = "|date_time| date_time.to_string()"))]
    pub created: DateTime<Utc>,
}

/// Contains a pair of local link addresses that are paired with a specific wireguard
/// public/private keypair.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssociatedAddresses {
    pub ipv4_address: ipnetwork::Ipv4Network,
    pub ipv6_address: ipnetwork::Ipv6Network,
}

/// Event that is emitted when the daemon has finished generating a key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum KeygenEvent {
    NewKey(PublicKey),
    TooManyKeys,
    GenerationFailure,
}

impl fmt::Display for KeygenEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            KeygenEvent::NewKey(new_key) => write!(f, "New wireguard key {}", new_key.key),
            KeygenEvent::TooManyKeys => write!(f, "Account has too many keys already"),
            KeygenEvent::GenerationFailure => write!(f, "Failed to generate new wireguard key"),
        }
    }
}
