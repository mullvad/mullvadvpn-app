#![allow(clippy::identity_op)]
use chrono::{offset::Utc, DateTime};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Deserializer, Serialize};
use std::{convert::TryFrom, fmt, time::Duration};
use talpid_types::net::wireguard;

pub const MIN_ROTATION_INTERVAL: Duration = Duration::from_secs(1 * 24 * 60 * 60);
pub const MAX_ROTATION_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
pub const DEFAULT_ROTATION_INTERVAL: Duration = if cfg!(target_os = "android") {
    Duration::from_secs(4 * 24 * 60 * 60)
} else {
    Duration::from_secs(7 * 24 * 60 * 60)
};

/// Whether to enable or disable quantum resistant tunnels when the setting
/// is set to `QuantumResistantState::Auto`.
const QUANTUM_RESISTANT_AUTO_STATE: bool = false;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuantumResistantState {
    Auto,
    On,
    Off,
}

/// Contains account specific wireguard data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub enum RotationIntervalError {
    TooSmall,
    TooLarge,
}

impl fmt::Display for RotationIntervalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RotationIntervalError::*;

        match *self {
            TooSmall => write!(
                f,
                "Rotation interval must be at least {} hours",
                MIN_ROTATION_INTERVAL.as_secs() / 60 / 60
            ),
            TooLarge => write!(
                f,
                "Rotation interval must be at most {} hours",
                MAX_ROTATION_INTERVAL.as_secs() / 60 / 60
            ),
        }
    }
}

impl std::error::Error for RotationIntervalError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RotationInterval(Duration);

impl RotationInterval {
    pub fn new(interval: Duration) -> Result<RotationInterval, RotationIntervalError> {
        if interval < MIN_ROTATION_INTERVAL {
            Err(RotationIntervalError::TooSmall)
        } else if interval > MAX_ROTATION_INTERVAL {
            Err(RotationIntervalError::TooLarge)
        } else {
            Ok(RotationInterval(interval))
        }
    }

    pub fn as_duration(&self) -> &Duration {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RotationInterval {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ivl = <Duration>::deserialize(deserializer)?;
        RotationInterval::new(ivl).map_err(|_error| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Other("Duration"),
                &"interval within allowed range",
            )
        })
    }
}

impl TryFrom<Duration> for RotationInterval {
    type Error = RotationIntervalError;

    fn try_from(duration: Duration) -> Result<RotationInterval, RotationIntervalError> {
        RotationInterval::new(duration)
    }
}

impl From<RotationInterval> for Duration {
    fn from(interval: RotationInterval) -> Duration {
        *interval.as_duration()
    }
}

impl Default for RotationInterval {
    fn default() -> RotationInterval {
        RotationInterval::new(DEFAULT_ROTATION_INTERVAL).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(
    target_os = "android",
    jnix(class_name = "net.mullvad.mullvadvpn.model.WireguardTunnelOptions")
)]
pub struct TunnelOptions {
    /// MTU for the wireguard tunnel
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|maybe_mtu| maybe_mtu.map(|mtu| mtu as i32)")
    )]
    pub mtu: Option<u16>,
    /// Temporary switch for wireguard-nt
    #[cfg(windows)]
    #[serde(rename = "wireguard_nt")]
    pub use_wireguard_nt: bool,
    /// Obtain a PSK using the relay config client.
    #[cfg_attr(
        target_os = "android",
        jnix(map = "|state| match state {
            QuantumResistantState::Auto => None,
            QuantumResistantState::On => Some(true),
            QuantumResistantState::Off => Some(false),
        }")
    )]
    pub quantum_resistant: QuantumResistantState,
    /// Interval used for automatic key rotation
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub rotation_interval: Option<RotationInterval>,
}

#[allow(clippy::derivable_impls)]
impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            mtu: None,
            quantum_resistant: QuantumResistantState::Auto,
            #[cfg(windows)]
            use_wireguard_nt: true,
            rotation_interval: None,
        }
    }
}

impl TunnelOptions {
    pub fn into_talpid_tunnel_options(self) -> wireguard::TunnelOptions {
        wireguard::TunnelOptions {
            mtu: self.mtu,
            #[cfg(windows)]
            use_wireguard_nt: self.use_wireguard_nt,
            quantum_resistant: match self.quantum_resistant {
                QuantumResistantState::Auto => QUANTUM_RESISTANT_AUTO_STATE,
                QuantumResistantState::On => true,
                QuantumResistantState::Off => false,
            },
        }
    }
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssociatedAddresses {
    pub ipv4_address: ipnetwork::Ipv4Network,
    pub ipv6_address: ipnetwork::Ipv6Network,
}
