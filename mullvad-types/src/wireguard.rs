#![allow(clippy::identity_op)]
use chrono::{DateTime, offset::Utc};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use serde::{Deserialize, Serialize};
use std::{fmt, net::IpAddr, str::FromStr};
use talpid_types::net::wireguard;

use crate::Intersection;

#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, PartialEq, Eq, Intersection)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum QuantumResistantState {
    #[default]
    On,
    Off,
}

impl QuantumResistantState {
    pub fn enabled(&self) -> bool {
        use QuantumResistantState::*;
        match self {
            On => true,
            Off => false,
        }
    }
}

impl fmt::Display for QuantumResistantState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QuantumResistantState::*;
        match self {
            On => f.write_str("on"),
            Off => f.write_str("off"),
        }
    }
}

impl FromStr for QuantumResistantState {
    type Err = QuantumResistantStateParseError;

    fn from_str(s: &str) -> Result<QuantumResistantState, Self::Err> {
        use QuantumResistantState::*;
        match s {
            "on" => Ok(On),
            "off" => Ok(Off),
            _ => Err(QuantumResistantStateParseError),
        }
    }
}

/// Returned when `QuantumResistantState::from_str` fails to convert a string into a
/// [`QuantumResistantState`] object.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid state")]
pub struct QuantumResistantStateParseError;

/// The WireGuard identity of this installation.
///
/// This is the pair of a private key and the tunnel addresses that Mullvad has assigned to the
/// corresponding public key. Both are found in a WireGuard config file generated at mullvad.net,
/// and they are all that is needed to establish a tunnel to a Mullvad relay. The app never talks
/// to any account service; the identity is created by importing such a config file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WireguardIdentity {
    pub private_key: wireguard::PrivateKey,
    /// The IPv4 address assigned to the key (`Address` in the config file).
    pub ipv4_address: Ipv4Network,
    /// The IPv6 address assigned to the key, if the config file contained one.
    #[serde(default)]
    pub ipv6_address: Option<Ipv6Network>,
    /// When the identity was imported.
    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,
}

impl WireguardIdentity {
    /// Create an identity from a private key and a list of tunnel addresses. The first IPv4 and
    /// the first IPv6 address in `addresses` are used.
    pub fn new(
        private_key: wireguard::PrivateKey,
        addresses: &[IpNetwork],
    ) -> Result<Self, ConfigParseError> {
        let ipv4_address = addresses
            .iter()
            .find_map(|net| match net {
                IpNetwork::V4(net) => Some(*net),
                IpNetwork::V6(_) => None,
            })
            .ok_or(ConfigParseError::MissingIpv4Address)?;
        let ipv6_address = addresses.iter().find_map(|net| match net {
            IpNetwork::V6(net) => Some(*net),
            IpNetwork::V4(_) => None,
        });
        Ok(Self {
            private_key,
            ipv4_address,
            ipv6_address,
            created: Utc::now(),
        })
    }

    /// Get the public key that corresponds to the private key.
    pub fn get_public_key(&self) -> PublicKey {
        PublicKey {
            key: self.private_key.public_key(),
            created: self.created,
        }
    }

    /// The tunnel addresses to configure on the tunnel interface.
    pub fn tunnel_addresses(&self) -> Vec<IpAddr> {
        let mut addresses = vec![IpAddr::V4(self.ipv4_address.ip())];
        if let Some(ipv6) = self.ipv6_address {
            addresses.push(IpAddr::V6(ipv6.ip()));
        }
        addresses
    }

    /// Parse the `[Interface]` section of a WireGuard config file, as generated at mullvad.net,
    /// into an identity. The `[Peer]` section, `DNS` and all other fields are ignored, since the
    /// app selects relays, DNS servers and so on by itself.
    pub fn from_config(config: &str) -> Result<Self, ConfigParseError> {
        #[derive(PartialEq, Eq)]
        enum Section {
            None,
            Interface,
            Other,
        }

        let mut section = Section::None;
        let mut seen_interface = false;
        let mut private_key = None;
        let mut addresses: Vec<IpNetwork> = vec![];

        for (index, raw_line) in config.lines().enumerate() {
            let line_number = index + 1;
            // `#` starts a comment, also in the middle of a line.
            let line = raw_line.split('#').next().unwrap_or_default().trim();
            // Tolerate a byte order mark at the start of the file.
            let line = line.trim_start_matches('\u{feff}');
            if line.is_empty() {
                continue;
            }

            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = if name.trim().eq_ignore_ascii_case("interface") {
                    if seen_interface {
                        return Err(ConfigParseError::MultipleInterfaces);
                    }
                    seen_interface = true;
                    Section::Interface
                } else {
                    Section::Other
                };
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                return Err(ConfigParseError::MalformedLine { line: line_number });
            };
            if section != Section::Interface {
                continue;
            }
            let value = value.trim();
            match key.trim().to_ascii_lowercase().as_str() {
                "privatekey" => {
                    if private_key.is_some() {
                        return Err(ConfigParseError::DuplicatePrivateKey);
                    }
                    private_key = Some(
                        wireguard::PrivateKey::from_base64(value)
                            .map_err(|_| ConfigParseError::InvalidPrivateKey)?,
                    );
                }
                "address" => {
                    for part in value.split(',').map(str::trim).filter(|p| !p.is_empty()) {
                        addresses.push(parse_address(part)?);
                    }
                }
                _ => (),
            }
        }

        if !seen_interface {
            return Err(ConfigParseError::MissingInterface);
        }
        let private_key = private_key.ok_or(ConfigParseError::MissingPrivateKey)?;
        if addresses.is_empty() {
            return Err(ConfigParseError::MissingAddress);
        }
        Self::new(private_key, &addresses)
    }
}

/// Parse `a.b.c.d/len` or a bare IP address, which implies a host prefix.
fn parse_address(address: &str) -> Result<IpNetwork, ConfigParseError> {
    if let Ok(network) = IpNetwork::from_str(address) {
        return Ok(network);
    }
    IpAddr::from_str(address)
        .map(IpNetwork::from)
        .map_err(|_| ConfigParseError::InvalidAddress(address.to_owned()))
}

/// Errors when interpreting a WireGuard config file as a [`WireguardIdentity`].
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ConfigParseError {
    #[error("The config has no [Interface] section")]
    MissingInterface,
    #[error("The config has more than one [Interface] section")]
    MultipleInterfaces,
    #[error("The config has no PrivateKey")]
    MissingPrivateKey,
    #[error("The config has more than one PrivateKey")]
    DuplicatePrivateKey,
    #[error("The PrivateKey is not a valid base64 encoded WireGuard key")]
    InvalidPrivateKey,
    #[error("The config has no Address")]
    MissingAddress,
    #[error("The config has no IPv4 Address")]
    MissingIpv4Address,
    #[error("Invalid Address: {0}")]
    InvalidAddress(String),
    #[error("Line {line} is not of the form 'key = value'")]
    MalformedLine { line: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TunnelOptions {
    /// MTU for the wireguard tunnel
    pub mtu: Option<u16>,
    /// Obtain a PSK using the relay config client.
    pub quantum_resistant: QuantumResistantState,
    /// Configure DAITA
    pub daita: bool,
    /// Use userspace WireGuard.
    pub userspace: bool,
}

#[expect(clippy::derivable_impls)]
impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            mtu: None,
            quantum_resistant: QuantumResistantState::default(),
            daita: false,
            userspace: false,
        }
    }
}

impl TunnelOptions {
    pub fn into_talpid_tunnel_options(self) -> wireguard::TunnelOptions {
        wireguard::TunnelOptions {
            mtu: self.mtu,
            quantum_resistant: self.quantum_resistant.enabled(),
            daita: self.daita,
            userspace: self.userspace,
        }
    }
}

/// Represents the public key of the imported WireGuard identity
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublicKey {
    pub key: wireguard::PublicKey,
    pub created: DateTime<Utc>,
}

/// What the daemon tells its clients about the current [`WireguardIdentity`].
/// This never includes the private key.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WireguardIdentityInfo {
    pub public_key: PublicKey,
    pub ipv4_address: Ipv4Network,
    pub ipv6_address: Option<Ipv6Network>,
}

impl From<&WireguardIdentity> for WireguardIdentityInfo {
    fn from(identity: &WireguardIdentity) -> Self {
        WireguardIdentityInfo {
            public_key: identity.get_public_key(),
            ipv4_address: identity.ipv4_address,
            ipv6_address: identity.ipv6_address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config(private_key: &str) -> String {
        format!(
            "[Interface]\n# Device: Fancy Seagull\nPrivateKey = {private_key}\n\
             Address = 10.66.185.96/32,fc00:bbbb:bbbb:bb01::3:b960/128\nDNS = 10.64.0.1\n\n\
             [Peer]\nPublicKey = 8UVwpDWbYIdcP/rLZXtZBoSKvQXLbFVxxJyyvVefWFs=\n\
             AllowedIPs = 0.0.0.0/0,::0/0\nEndpoint = 193.32.127.66:51820\n"
        )
    }

    fn random_key() -> (wireguard::PrivateKey, String) {
        let key = wireguard::PrivateKey::new_from_random();
        let encoded = key.to_base64();
        (key, encoded)
    }

    #[test]
    fn parses_mullvad_config() {
        let (key, encoded) = random_key();
        let identity = WireguardIdentity::from_config(&sample_config(&encoded)).unwrap();
        assert_eq!(identity.private_key.public_key(), key.public_key());
        assert_eq!(identity.ipv4_address, "10.66.185.96/32".parse().unwrap());
        assert_eq!(
            identity.ipv6_address,
            Some("fc00:bbbb:bbbb:bb01::3:b960/128".parse().unwrap())
        );
        assert_eq!(
            identity.tunnel_addresses(),
            vec![
                "10.66.185.96".parse::<IpAddr>().unwrap(),
                "fc00:bbbb:bbbb:bb01::3:b960".parse::<IpAddr>().unwrap(),
            ]
        );
    }

    #[test]
    fn tolerates_case_whitespace_crlf_and_multiple_address_lines() {
        let (_, encoded) = random_key();
        let config = format!(
            "\u{feff}[interface]\r\n  privatekey=  {encoded} # trailing comment\r\n\
             ADDRESS = 10.1.2.3\r\nAddress = fd00::1/128\r\n"
        );
        let identity = WireguardIdentity::from_config(&config).unwrap();
        assert_eq!(identity.ipv4_address, "10.1.2.3/32".parse().unwrap());
        assert_eq!(identity.ipv6_address, Some("fd00::1/128".parse().unwrap()));
    }

    #[test]
    fn ipv6_is_optional() {
        let (_, encoded) = random_key();
        let config = format!("[Interface]\nPrivateKey = {encoded}\nAddress = 10.1.2.3/32\n");
        let identity = WireguardIdentity::from_config(&config).unwrap();
        assert_eq!(identity.ipv6_address, None);
        assert_eq!(identity.tunnel_addresses().len(), 1);
    }

    #[test]
    fn rejects_invalid_configs() {
        let (_, encoded) = random_key();
        let cases: Vec<(String, ConfigParseError)> = vec![
            (String::new(), ConfigParseError::MissingInterface),
            (
                "[Peer]\nPublicKey = x\n".into(),
                ConfigParseError::MissingInterface,
            ),
            (
                "[Interface]\nAddress = 10.0.0.2/32\n".into(),
                ConfigParseError::MissingPrivateKey,
            ),
            (
                "[Interface]\nPrivateKey = not-a-key\nAddress = 10.0.0.2/32\n".into(),
                ConfigParseError::InvalidPrivateKey,
            ),
            (
                format!("[Interface]\nPrivateKey = {encoded}\n"),
                ConfigParseError::MissingAddress,
            ),
            (
                format!("[Interface]\nPrivateKey = {encoded}\nAddress = fd00::1/128\n"),
                ConfigParseError::MissingIpv4Address,
            ),
            (
                format!("[Interface]\nPrivateKey = {encoded}\nAddress = banana\n"),
                ConfigParseError::InvalidAddress("banana".into()),
            ),
            (
                format!("[Interface]\nPrivateKey = {encoded}\nbroken line\n"),
                ConfigParseError::MalformedLine { line: 3 },
            ),
            (
                format!("[Interface]\nPrivateKey = {encoded}\n[Interface]\n"),
                ConfigParseError::MultipleInterfaces,
            ),
        ];
        for (config, expected) in cases {
            assert_eq!(
                WireguardIdentity::from_config(&config).unwrap_err(),
                expected,
                "config: {config:?}"
            );
        }
    }

    #[test]
    fn identity_roundtrips_through_json() {
        let (_, encoded) = random_key();
        let identity = WireguardIdentity::from_config(&sample_config(&encoded)).unwrap();
        let json = serde_json::to_string(&identity).unwrap();
        let back: WireguardIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(identity, back);
    }
}
