use crate::{
    relay_constraints::{
        BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint,
        ObfuscationSettings, RelayConstraints, RelaySettings, RelaySettingsUpdate,
        SelectedObfuscation, WireguardConstraints,
    },
    wireguard,
};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(target_os = "windows")]
use std::{collections::HashSet, path::PathBuf};
use talpid_types::net::{openvpn, GenericTunnelOptions};

mod dns;

/// The version used by the current version of the code. Should always be the
/// latest version that exists in `SettingsVersion`.
/// This should be bumped when a new version is introduced along with a migration
/// being added to `mullvad-daemon`.
pub const CURRENT_SETTINGS_VERSION: SettingsVersion = SettingsVersion::V6;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
#[repr(u32)]
pub enum SettingsVersion {
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Settings {
    relay_settings: RelaySettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub bridge_settings: BridgeSettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub obfuscation_settings: ObfuscationSettings,
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
    pub show_beta_releases: bool,
    /// Split tunneling settings
    #[cfg(windows)]
    pub split_tunnel: SplitTunnelSettings,
    /// Temporary variable for a random number between 0 and 1 that determines if the user should
    /// use wireguard or openvpn when the automatic feature is set. This variable will be removed
    /// in future versions.
    /// A value of -1.0 implies that the variable should be initialized to a random number.
    /// NOTE: This field will be removed completely in future versions.
    #[serde(default = "out_of_range_wg_migration_rand_num")]
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub wg_migration_rand_num: f32,
    /// Specifies settings schema version
    #[cfg_attr(target_os = "android", jnix(skip))]
    settings_version: SettingsVersion,
}

fn out_of_range_wg_migration_rand_num() -> f32 {
    -1.0
}

#[cfg(windows)]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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
                location: Constraint::Only(LocationConstraint::Country("se".to_owned())),
                wireguard_constraints: WireguardConstraints {
                    entry_location: Constraint::Only(LocationConstraint::Country("se".to_owned())),
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
            wg_migration_rand_num: rand::thread_rng().gen_range(0.0..=1.0),
            #[cfg(windows)]
            split_tunnel: SplitTunnelSettings::default(),
            settings_version: CURRENT_SETTINGS_VERSION,
        }
    }
}

impl Settings {
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
            log::debug!(
                "Changing relay settings:\n\tfrom: {}\n\tto: {}",
                self.relay_settings,
                new_settings
            );

            self.relay_settings = new_settings;
            true
        } else {
            false
        }
    }

    pub fn get_bridge_state(&self) -> BridgeState {
        self.bridge_state
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> bool {
        if self.bridge_state != bridge_state {
            self.bridge_state = bridge_state;
            true
        } else {
            false
        }
    }

    pub fn get_settings_version(&self) -> SettingsVersion {
        self.settings_version
    }
}

/// TunnelOptions holds configuration data that applies to all kinds of tunnels.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
