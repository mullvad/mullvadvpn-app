use crate::{
    access_method,
    constraints::Constraint,
    custom_list::CustomListsSettings,
    relay_constraints::{
        BridgeSettings, BridgeState, GeographicLocationConstraint, LocationConstraint,
        ObfuscationSettings, RelayConstraints, RelayOverride, RelaySettings,
        RelaySettingsFormatter, SelectedObfuscation, WireguardConstraints,
    },
    wireguard,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(any(windows, target_os = "android", target_os = "macos"))]
use std::collections::HashSet;
use talpid_types::net::{openvpn, GenericTunnelOptions};

mod dns;

/// The version used by the current version of the code. Should always be the
/// latest version that exists in `SettingsVersion`.
/// This should be bumped when a new version is introduced along with a migration
/// being added to `mullvad-daemon`.
pub const CURRENT_SETTINGS_VERSION: SettingsVersion = SettingsVersion::V11;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
#[repr(u32)]
pub enum SettingsVersion {
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    V10 = 10,
    V11 = 11,
}

impl<'de> Deserialize<'de> for SettingsVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match <u32>::deserialize(deserializer)? {
            v if v == SettingsVersion::V2 as u32 => Ok(SettingsVersion::V2),
            v if v == SettingsVersion::V3 as u32 => Ok(SettingsVersion::V3),
            v if v == SettingsVersion::V4 as u32 => Ok(SettingsVersion::V4),
            v if v == SettingsVersion::V5 as u32 => Ok(SettingsVersion::V5),
            v if v == SettingsVersion::V6 as u32 => Ok(SettingsVersion::V6),
            v if v == SettingsVersion::V7 as u32 => Ok(SettingsVersion::V7),
            v if v == SettingsVersion::V8 as u32 => Ok(SettingsVersion::V8),
            v if v == SettingsVersion::V9 as u32 => Ok(SettingsVersion::V9),
            v if v == SettingsVersion::V10 as u32 => Ok(SettingsVersion::V10),
            v if v == SettingsVersion::V11 as u32 => Ok(SettingsVersion::V11),
            v => Err(serde::de::Error::custom(format!(
                "{v} is not a valid SettingsVersion"
            ))),
        }
    }
}

impl Serialize for SettingsVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(*self as u32)
    }
}

/// Mullvad daemon settings.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub relay_settings: RelaySettings,
    pub bridge_settings: BridgeSettings,
    pub obfuscation_settings: ObfuscationSettings,
    pub bridge_state: BridgeState,
    /// All of the custom relay lists
    pub custom_lists: CustomListsSettings,
    /// API access methods
    pub api_access_methods: access_method::Settings,
    /// If the daemon should allow communication with private (LAN) networks.
    pub allow_lan: bool,
    /// Extra level of kill switch. When this setting is on, the disconnected state will block
    /// the firewall to not allow any traffic in or out.
    #[cfg(not(target_os = "android"))]
    pub block_when_disconnected: bool,
    /// If the daemon should connect the VPN tunnel directly on start or not.
    pub auto_connect: bool,
    /// Options that should be applied to tunnels of a specific type regardless of where the relays
    /// might be located.
    pub tunnel_options: TunnelOptions,
    /// Overrides for relays
    pub relay_overrides: Vec<RelayOverride>,
    /// Whether to notify users of beta updates.
    pub show_beta_releases: bool,
    /// Split tunneling settings
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    pub split_tunnel: SplitTunnelSettings,
    /// Specifies settings schema version
    pub settings_version: SettingsVersion,
}

#[cfg(any(windows, target_os = "android", target_os = "macos"))]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct SplitTunnelSettings {
    /// Toggles split tunneling on or off
    pub enable_exclusions: bool,
    /// Set of applications to exclude from the tunnel.
    pub apps: HashSet<SplitApp>,
}

/// An application whose traffic should be excluded from any active tunnel.
#[cfg(any(windows, target_os = "macos"))]
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SplitApp(std::path::PathBuf);

/// An application whose traffic should be excluded from any active tunnel.
#[cfg(target_os = "android")]
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SplitApp(String);

#[cfg(any(windows, target_os = "macos"))]
impl SplitApp {
    /// Convert the underlying path to a [`String`].
    /// This function will fail if the underlying path string is not valid UTF-8. See
    /// [`std::ffi::OsStr::to_str`] for details.
    pub fn to_string(self) -> Option<String> {
        self.0.as_os_str().to_str().map(str::to_string)
    }

    /// This is the String-representation as expected by [`SetExcludedApps`].
    pub fn to_tunnel_command_repr(self) -> std::ffi::OsString {
        self.0.as_os_str().to_owned()
    }

    pub fn display(&self) -> std::path::Display<'_> {
        self.0.display()
    }
}

#[cfg(target_os = "android")]
impl SplitApp {
    /// Convert the underlying app name to a [`String`].
    ///
    /// # Note
    /// This function is fallible due to the Window's dito being fallible, and it is convenient to
    /// have the same API across all platforms.
    pub fn to_string(self) -> Option<String> {
        Some(self.0)
    }

    /// This is the String-representation as expected by [`SetExcludedApps`].
    pub fn to_tunnel_command_repr(self) -> String {
        self.0
    }
}

#[cfg(any(windows, target_os = "macos"))]
impl From<String> for SplitApp {
    fn from(value: String) -> Self {
        SplitApp::from(std::path::PathBuf::from(value))
    }
}

#[cfg(any(windows, target_os = "macos"))]
impl From<std::path::PathBuf> for SplitApp {
    fn from(value: std::path::PathBuf) -> Self {
        SplitApp(value)
    }
}

#[cfg(target_os = "android")]
impl From<String> for SplitApp {
    fn from(value: String) -> Self {
        SplitApp(value)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            relay_settings: RelaySettings::Normal(RelayConstraints {
                location: Constraint::Only(LocationConstraint::Location(
                    GeographicLocationConstraint::Country("se".to_owned()),
                )),
                wireguard_constraints: WireguardConstraints {
                    entry_location: Constraint::Only(LocationConstraint::Location(
                        GeographicLocationConstraint::Country("se".to_owned()),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            }),
            bridge_settings: BridgeSettings::default(),
            obfuscation_settings: ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Auto,
                ..Default::default()
            },
            bridge_state: BridgeState::Auto,
            custom_lists: CustomListsSettings::default(),
            api_access_methods: access_method::Settings::default(),
            allow_lan: false,
            #[cfg(not(target_os = "android"))]
            block_when_disconnected: false,
            auto_connect: false,
            tunnel_options: TunnelOptions::default(),
            relay_overrides: vec![],
            show_beta_releases: false,
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            split_tunnel: SplitTunnelSettings::default(),
            settings_version: CURRENT_SETTINGS_VERSION,
        }
    }
}

impl Settings {
    pub fn get_relay_settings(&self) -> RelaySettings {
        self.relay_settings.clone()
    }

    pub fn set_relay_settings(&mut self, new_settings: RelaySettings) {
        if self.relay_settings != new_settings {
            if !new_settings.supports_bridge() && BridgeState::On == self.bridge_state {
                self.bridge_state = BridgeState::Auto;
            }

            log::debug!(
                "Changing relay settings:\n\tfrom: {}\n\tto: {}",
                RelaySettingsFormatter {
                    settings: &self.relay_settings,
                    custom_lists: &self.custom_lists,
                },
                RelaySettingsFormatter {
                    settings: &new_settings,
                    custom_lists: &self.custom_lists,
                },
            );

            self.relay_settings = new_settings;
        }
    }

    pub fn set_relay_override(&mut self, relay_override: RelayOverride) {
        let existing_override = self
            .relay_overrides
            .iter_mut()
            .enumerate()
            .find(|(_, elem)| elem.hostname == relay_override.hostname);
        match existing_override {
            None => self.relay_overrides.push(relay_override),
            Some((index, elem)) => {
                if relay_override.is_empty() {
                    self.relay_overrides.swap_remove(index);
                } else {
                    *elem = relay_override;
                }
            }
        }
    }
}

/// TunnelOptions holds configuration data that applies to all kinds of tunnels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TunnelOptions {
    /// openvpn holds OpenVPN specific tunnel options.
    pub openvpn: openvpn::TunnelOptions,
    /// Contains wireguard tunnel options.
    pub wireguard: wireguard::TunnelOptions,
    /// Contains generic tunnel options that may apply to more than a single tunnel type.
    pub generic: GenericTunnelOptions,
    /// DNS options.
    pub dns_options: DnsOptions,
}

pub use dns::{CustomDnsOptions, DefaultDnsOptions, DnsOptions, DnsState};

impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            openvpn: openvpn::TunnelOptions::default(),
            wireguard: wireguard::TunnelOptions::default(),
            generic: GenericTunnelOptions {
                // Enable IPv6 by default on Android and macOS
                enable_ipv6: cfg!(target_os = "android") || cfg!(target_os = "macos"),
            },
            dns_options: DnsOptions::default(),
        }
    }
}
