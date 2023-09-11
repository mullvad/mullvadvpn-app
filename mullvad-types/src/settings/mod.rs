use crate::{
    api_access_method,
    custom_list::CustomListsSettings,
    relay_constraints::{
        BridgeConstraints, BridgeSettings, BridgeState, Constraint, GeographicLocationConstraint,
        LocationConstraint, ObfuscationSettings, RelayConstraints, RelaySettings,
        RelaySettingsFormatter, RelaySettingsUpdate, SelectedObfuscation, WireguardConstraints,
    },
    wireguard,
};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(target_os = "windows")]
use std::{collections::HashSet, path::PathBuf};
use talpid_types::net::{openvpn, GenericTunnelOptions};

mod dns;

/// The version used by the current version of the code. Should always be the
/// latest version that exists in `SettingsVersion`.
/// This should be bumped when a new version is introduced along with a migration
/// being added to `mullvad-daemon`.
pub const CURRENT_SETTINGS_VERSION: SettingsVersion = SettingsVersion::V7;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
#[repr(u32)]
pub enum SettingsVersion {
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
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
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Settings {
    pub relay_settings: RelaySettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub bridge_settings: BridgeSettings,
    pub obfuscation_settings: ObfuscationSettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub bridge_state: BridgeState,
    /// All of the custom relay lists
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub custom_lists: CustomListsSettings,
    /// API access methods.
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub api_access_methods: api_access_method::Settings,
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
    pub show_beta_releases: bool,
    /// Split tunneling settings
    #[cfg(windows)]
    pub split_tunnel: SplitTunnelSettings,
    /// Specifies settings schema version
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub settings_version: SettingsVersion,
}

#[cfg(windows)]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct SplitTunnelSettings {
    /// Toggles split tunneling on or off
    pub enable_exclusions: bool,
    /// List of applications to exclude from the tunnel.
    pub apps: HashSet<PathBuf>,
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
            bridge_settings: BridgeSettings::Normal(BridgeConstraints::default()),
            obfuscation_settings: ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Off,
                ..Default::default()
            },
            bridge_state: BridgeState::Auto,
            allow_lan: false,
            block_when_disconnected: false,
            auto_connect: false,
            tunnel_options: TunnelOptions::default(),
            show_beta_releases: false,
            #[cfg(windows)]
            split_tunnel: SplitTunnelSettings::default(),
            settings_version: CURRENT_SETTINGS_VERSION,
            custom_lists: CustomListsSettings::default(),
            api_access_methods: api_access_method::Settings::default(),
        }
    }
}

impl Settings {
    pub fn get_relay_settings(&self) -> RelaySettings {
        self.relay_settings.clone()
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) {
        let update_supports_bridge = update.supports_bridge();
        let new_settings = self.relay_settings.merge(update);
        if self.relay_settings != new_settings {
            if !update_supports_bridge && BridgeState::On == self.bridge_state {
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
}

/// TunnelOptions holds configuration data that applies to all kinds of tunnels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                // Enable IPv6 be default on Android
                enable_ipv6: cfg!(target_os = "android"),
            },
            dns_options: DnsOptions::default(),
        }
    }
}
