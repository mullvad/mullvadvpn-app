use crate::relay_constraints::{
    BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint,
    RelayConstraints, RelaySettings, RelaySettingsUpdate,
};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json;
use std::net::IpAddr;
use talpid_types::net::{openvpn, wireguard, GenericTunnelOptions};

mod migrations;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Malformed settings")]
    ParseError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to read any version of the settings")]
    NoMatchingVersion,
}


/// Mullvad daemon settings.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Settings {
    account_token: Option<String>,
    relay_settings: RelaySettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub bridge_settings: BridgeSettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    bridge_state: BridgeState,
    /// If the daemon should allow communication with private (LAN) networks.
    pub allow_lan: bool,
    /// Extra level of kill switch. When this setting is on, the disconnected state will block
    /// the firewall to not allow any traffic in or out.
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub block_when_disconnected: bool,
    /// If the daemon should connect the VPN tunnel directly on start or not.
    pub auto_connect: bool,
    /// Options that should be applied to tunnels of a specific type regardless of where the relays
    /// might be located.
    pub tunnel_options: TunnelOptions,
    /// Whether to notify users of beta updates.
    #[serde(deserialize_with = "deserialize_show_beta_releases")]
    pub show_beta_releases: bool,
    /// Specifies settings schema version
    #[cfg_attr(target_os = "android", jnix(skip))]
    settings_version: migrations::SettingsVersion,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            account_token: None,
            relay_settings: RelaySettings::Normal(RelayConstraints {
                location: Constraint::Only(LocationConstraint::Country("se".to_owned())),
                ..Default::default()
            }),
            bridge_settings: BridgeSettings::Normal(BridgeConstraints::default()),
            bridge_state: BridgeState::Auto,
            allow_lan: false,
            block_when_disconnected: false,
            auto_connect: false,
            tunnel_options: TunnelOptions::default(),
            show_beta_releases: false,
            settings_version: migrations::SettingsVersion::V2,
        }
    }
}

impl Settings {
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Error::ParseError)
    }

    pub fn migrate_from_bytes(bytes: &[u8]) -> Result<Self> {
        migrations::try_migrate_settings(&bytes)
    }

    pub fn get_account_token(&self) -> Option<String> {
        self.account_token.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, mut account_token: Option<String>) -> bool {
        if account_token.as_ref().map(String::len) == Some(0) {
            debug!("Setting empty account token is treated as unsetting it");
            account_token = None;
        }
        if account_token != self.account_token {
            if account_token.is_none() {
                info!("Unsetting account token");
            } else if self.account_token.is_none() {
                info!("Setting account token");
            } else {
                info!("Changing account token")
            }
            self.account_token = account_token;
            true
        } else {
            false
        }
    }

    pub fn get_relay_settings(&self) -> RelaySettings {
        self.relay_settings.clone()
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> bool {
        let update_supports_bridge = update.supports_bridge();
        let new_settings = self.relay_settings.merge(update);
        if self.relay_settings != new_settings {
            if !update_supports_bridge && BridgeState::On == self.bridge_state {
                self.bridge_state = BridgeState::Auto;
            }
            debug!(
                "Changing relay settings:\n\tfrom: {}\n\tto: {}",
                self.relay_settings, new_settings
            );

            self.relay_settings = new_settings;
            true
        } else {
            false
        }
    }

    pub fn get_bridge_state(&self) -> &BridgeState {
        &self.bridge_state
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> bool {
        if self.bridge_state != bridge_state {
            self.bridge_state = bridge_state;
            if self.bridge_state == BridgeState::On {
                self.relay_settings.ensure_bridge_compatibility();
            }
            true
        } else {
            false
        }
    }
}

/// TunnelOptions holds configuration data that applies to all kinds of tunnels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct TunnelOptions {
    /// openvpn holds OpenVPN specific tunnel options.
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub openvpn: openvpn::TunnelOptions,
    /// Contains wireguard tunnel options.
    pub wireguard: wireguard::TunnelOptions,
    /// Contains generic tunnel options that may apply to more than a single tunnel type.
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub generic: GenericTunnelOptions,
    /// Custom DNS options.
    pub dns_options: DnsOptions,
}

/// Custom DNS config
#[serde(default)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct DnsOptions {
    /// Whether to use the addresses in `custom_dns`.
    pub custom: bool,
    /// Custom DNS servers to use.
    pub addresses: Vec<IpAddr>,
}

impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            openvpn: openvpn::TunnelOptions::default(),
            wireguard: wireguard::TunnelOptions {
                mtu: None,
                automatic_rotation: None,
            },
            generic: GenericTunnelOptions {
                // Enable IPv6 be default on Android
                enable_ipv6: cfg!(target_os = "android"),
            },
            dns_options: DnsOptions::default(),
        }
    }
}

/// Used to deserialize the `show_beta_releases` field in the settings struct, as it used to be
/// a nullable field, but it is no longer.
fn deserialize_show_beta_releases<'de, D: serde::de::Deserializer<'de>>(
    field: D,
) -> std::result::Result<bool, D::Error> {
    Option::deserialize(field).map(|value| value.unwrap_or(false))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialization_of_2020_4_format() {
        let old_settings = br#"{
              "account_token": "0000000000000000",
              "relay_settings": {
                "normal": {
                  "location": {
                    "only": {
                      "country": "gb"
                    }
                  },
                  "tunnel_protocol": {
                    "only": "wireguard"
                  },
                  "wireguard_constraints": {
                    "port": "any"
                  },
                  "openvpn_constraints": {
                    "port": "any",
                    "protocol": "any"
                  }
                }
              },
              "bridge_settings": {
                "normal": {
                  "location": "any"
                }
              },
              "bridge_state": "auto",
              "allow_lan": true,
              "block_when_disconnected": false,
              "auto_connect": true,
              "tunnel_options": {
                "openvpn": {
                  "mssfix": null
                },
                "wireguard": {
                  "mtu": null,
                  "automatic_rotation": null
                },
                "generic": {
                  "enable_ipv6": true
                }
              },
              "settings_version": 2,
              "show_beta_releases": null
        }"#;

        let _ = Settings::load_from_bytes(old_settings).unwrap();
    }
}
