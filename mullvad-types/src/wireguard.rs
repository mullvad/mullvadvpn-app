#![allow(clippy::identity_op)]
use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr, time::Duration};
use talpid_types::net::wireguard;

pub const MIN_ROTATION_INTERVAL: Duration = Duration::from_secs(1 * 24 * 60 * 60);
pub const MAX_ROTATION_INTERVAL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
pub const DEFAULT_ROTATION_INTERVAL: Duration = MAX_ROTATION_INTERVAL;

/// Whether to enable or disable quantum resistant tunnels when the setting is set to
/// `QuantumResistantState::Auto`. It is currently enabled by default on Linux but disabled on all
/// other platforms.
const QUANTUM_RESISTANT_AUTO_STATE: bool = cfg!(target_os = "linux");

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum QuantumResistantState {
    Auto,
    On,
    Off,
}

impl fmt::Display for QuantumResistantState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantumResistantState::Auto => f.write_str("auto"),
            QuantumResistantState::On => f.write_str("on"),
            QuantumResistantState::Off => f.write_str("off"),
        }
    }
}

impl FromStr for QuantumResistantState {
    type Err = QuantumResistantStateParseError;

    fn from_str(s: &str) -> Result<QuantumResistantState, Self::Err> {
        match s {
            "any" | "auto" => Ok(QuantumResistantState::Auto),
            "on" => Ok(QuantumResistantState::On),
            "off" => Ok(QuantumResistantState::Off),
            _ => Err(QuantumResistantStateParseError),
        }
    }
}

/// Returned when `QuantumResistantState::from_str` fails to convert a string into a
/// [`QuantumResistantState`] object.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid state")]
pub struct QuantumResistantStateParseError;

#[cfg(daita)]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaitaSettings {
    pub enabled: bool,
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

impl fmt::Display for RotationInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} hours", self.as_duration().as_secs() / 60 / 60)
    }
}

#[cfg(feature = "clap")]
impl clap::builder::ValueParserFactory for RotationInterval {
    type Parser = clap::builder::RangedU64ValueParser<RotationInterval>;

    fn value_parser() -> Self::Parser {
        clap::builder::RangedU64ValueParser::new().range(
            (MIN_ROTATION_INTERVAL.as_secs() / 60 / 60)
                ..(MAX_ROTATION_INTERVAL.as_secs() / 60 / 60),
        )
    }
}

impl TryFrom<u64> for RotationInterval {
    type Error = RotationIntervalError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        // Convert a u64, specified in hours, to a `RotationInterval`
        let val = value
            .checked_mul(60 * 60)
            .ok_or(RotationIntervalError::TooLarge)?;
        RotationInterval::new(Duration::from_secs(val))
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TunnelOptions {
    /// MTU for the wireguard tunnel
    pub mtu: Option<u16>,
    /// Obtain a PSK using the relay config client.
    pub quantum_resistant: QuantumResistantState,
    /// Configure DAITA
    #[cfg(daita)]
    pub daita: DaitaSettings,
    /// Interval used for automatic key rotation
    pub rotation_interval: Option<RotationInterval>,
}

#[allow(clippy::derivable_impls)]
impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            mtu: None,
            quantum_resistant: QuantumResistantState::Auto,
            #[cfg(daita)]
            daita: DaitaSettings::default(),
            rotation_interval: None,
        }
    }
}

impl TunnelOptions {
    pub fn into_talpid_tunnel_options(self) -> wireguard::TunnelOptions {
        wireguard::TunnelOptions {
            mtu: self.mtu,
            quantum_resistant: match self.quantum_resistant {
                QuantumResistantState::Auto => QUANTUM_RESISTANT_AUTO_STATE,
                QuantumResistantState::On => true,
                QuantumResistantState::Off => false,
            },
            #[cfg(daita)]
            daita: self.daita.enabled,
        }
    }
}

/// Represents a published public key
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublicKey {
    pub key: wireguard::PublicKey,
    pub created: DateTime<Utc>,
}

/// Contains a pair of local link addresses that are paired with a specific wireguard
/// public/private keypair.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssociatedAddresses {
    pub ipv4_address: ipnetwork::Ipv4Network,
    pub ipv6_address: ipnetwork::Ipv6Network,
}
