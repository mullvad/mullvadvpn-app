use crate::relay_constraints::{
    BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint,
    RelayConstraints, RelaySettings, RelaySettingsUpdate,
};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};
use talpid_types::{
    net::{openvpn, wireguard, GenericTunnelOptions},
    ErrorExt,
};

mod migrations;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Unable to create settings directory")]
    DirectoryError(#[error(source)] mullvad_paths::Error),

    #[error(display = "Unable to read settings from {}", _0)]
    ReadError(String, #[error(source)] io::Error),

    #[error(display = "Unable to remove settings file {}", _0)]
    DeleteError(String, #[error(source)] io::Error),

    #[error(display = "Malformed settings")]
    ParseError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to serialize settings to JSON")]
    SerializeError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to write settings to {}", _0)]
    WriteError(String, #[error(source)] io::Error),

    #[error(display = "Invalid OpenVPN proxy configuration: {}", _0)]
    InvalidProxyData(String),

    #[error(display = "Unable to read any version of the settings")]
    NoMatchingVersion,
}

static SETTINGS_FILE: &str = "settings.json";


/// Mullvad daemon settings.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Settings {
    account_token: Option<String>,
    relay_settings: RelaySettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    bridge_settings: BridgeSettings,
    #[cfg_attr(target_os = "android", jnix(skip))]
    bridge_state: BridgeState,
    /// If the daemon should allow communication with private (LAN) networks.
    allow_lan: bool,
    /// Extra level of kill switch. When this setting is on, the disconnected state will block
    /// the firewall to not allow any traffic in or out.
    #[cfg_attr(target_os = "android", jnix(skip))]
    block_when_disconnected: bool,
    /// If the daemon should connect the VPN tunnel directly on start or not.
    auto_connect: bool,
    /// Options that should be applied to tunnels of a specific type regardless of where the relays
    /// might be located.
    #[cfg_attr(target_os = "android", jnix(skip))]
    tunnel_options: TunnelOptions,
    /// Whether to notify users of beta updates.
    show_beta_releases: Option<bool>,
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
            bridge_settings: BridgeSettings::Normal(BridgeConstraints {
                location: Constraint::Any,
            }),
            bridge_state: BridgeState::Auto,
            allow_lan: false,
            block_when_disconnected: false,
            auto_connect: false,
            tunnel_options: TunnelOptions::default(),
            show_beta_releases: None,
            settings_version: migrations::SettingsVersion::V2,
        }
    }
}

impl Settings {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load() -> Result<Settings> {
        let path = Self::get_settings_path()?;
        match File::open(&path) {
            Ok(file) => {
                info!("Loading settings from {}", path.display());
                let mut settings_bytes = vec![];
                io::BufReader::new(file)
                    .read_to_end(&mut settings_bytes)
                    .map_err(|e| Error::ReadError("Failed to read settings file".to_owned(), e))?;
                Self::parse_settings(&mut settings_bytes).or_else(|e| {
                    log::error!(
                        "{}",
                        e.display_chain_with_msg("Failed to parse settings file")
                    );
                    let settings = migrations::try_migrate_settings(&settings_bytes)?;
                    if let Err(e) = settings.save() {
                        log::error!("Failed to save settings after migration: {}", e);
                    }
                    Ok(settings)
                })
            }
            Err(e) => Err(Error::ReadError(path.display().to_string(), e)),
        }
    }

    /// Serializes the settings and saves them to the file it was loaded from.
    fn save(&self) -> Result<()> {
        let path = Self::get_settings_path()?;

        debug!("Writing settings to {}", path.display());
        let mut file =
            File::create(&path).map_err(|e| Error::WriteError(path.display().to_string(), e))?;

        serde_json::to_writer_pretty(&mut file, self).map_err(Error::SerializeError)?;
        file.sync_all()
            .map_err(|e| Error::WriteError(path.display().to_string(), e))
    }

    /// Resets default settings
    pub fn reset(&mut self) -> Result<()> {
        *self = Default::default();
        self.save().or_else(|e| {
            log::error!(
                "{}",
                e.display_chain_with_msg("Unable to save default settings")
            );
            log::error!("Will attempt to remove settings file");
            Self::get_settings_path().and_then(|path| {
                fs::remove_file(&path)
                    .map_err(|e| Error::DeleteError(path.display().to_string(), e))
            })
        })
    }

    fn get_settings_path() -> Result<PathBuf> {
        let dir = ::mullvad_paths::settings_dir().map_err(Error::DirectoryError)?;
        Ok(dir.join(SETTINGS_FILE))
    }

    fn parse_settings(bytes: &[u8]) -> Result<Settings> {
        serde_json::from_slice(bytes).map_err(Error::ParseError)
    }

    pub fn get_account_token(&self) -> Option<String> {
        self.account_token.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, mut account_token: Option<String>) -> Result<bool> {
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
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_relay_settings(&self) -> RelaySettings {
        self.relay_settings.clone()
    }

    pub fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<bool> {
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
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_allow_lan(&self) -> bool {
        self.allow_lan
    }

    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<bool> {
        if allow_lan != self.allow_lan {
            self.allow_lan = allow_lan;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_block_when_disconnected(&self) -> bool {
        self.block_when_disconnected
    }

    pub fn set_block_when_disconnected(&mut self, block_when_disconnected: bool) -> Result<bool> {
        if block_when_disconnected != self.block_when_disconnected {
            self.block_when_disconnected = block_when_disconnected;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_auto_connect(&self) -> bool {
        self.auto_connect
    }

    pub fn set_auto_connect(&mut self, auto_connect: bool) -> Result<bool> {
        if auto_connect != self.auto_connect {
            self.auto_connect = auto_connect;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn set_openvpn_mssfix(&mut self, openvpn_mssfix: Option<u16>) -> Result<bool> {
        if self.tunnel_options.openvpn.mssfix != openvpn_mssfix {
            self.tunnel_options.openvpn.mssfix = openvpn_mssfix;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn set_enable_ipv6(&mut self, enable_ipv6: bool) -> Result<bool> {
        if self.tunnel_options.generic.enable_ipv6 != enable_ipv6 {
            self.tunnel_options.generic.enable_ipv6 = enable_ipv6;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<bool> {
        if self.tunnel_options.wireguard.mtu != mtu {
            self.tunnel_options.wireguard.mtu = mtu;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn set_wireguard_rotation_interval(
        &mut self,
        automatic_rotation: Option<u32>,
    ) -> Result<bool> {
        if self.tunnel_options.wireguard.automatic_rotation != automatic_rotation {
            self.tunnel_options.wireguard.automatic_rotation = automatic_rotation;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_tunnel_options(&self) -> &TunnelOptions {
        &self.tunnel_options
    }

    pub fn get_show_beta_releases(&self) -> Option<bool> {
        self.show_beta_releases.clone()
    }

    pub fn set_show_beta_releases(&mut self, enabled: bool) -> Result<bool> {
        if Some(enabled) != self.show_beta_releases {
            self.show_beta_releases = Some(enabled);
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_bridge_settings(&self) -> &BridgeSettings {
        &self.bridge_settings
    }

    pub fn set_bridge_settings(&mut self, bridge_settings: BridgeSettings) -> Result<bool> {
        if self.bridge_settings != bridge_settings {
            self.bridge_settings = bridge_settings;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_bridge_state(&self) -> &BridgeState {
        &self.bridge_state
    }

    pub fn set_bridge_state(&mut self, bridge_state: BridgeState) -> Result<bool> {
        if self.bridge_state != bridge_state {
            self.bridge_state = bridge_state;
            if self.bridge_state == BridgeState::On {
                self.relay_settings.ensure_bridge_compatibility();
            }
            self.save().map(|_| true)
        } else {
            Ok(false)
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
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub wireguard: wireguard::TunnelOptions,
    /// Contains generic tunnel options that may apply to more than a single tunnel type.
    pub generic: GenericTunnelOptions,
}

impl Default for TunnelOptions {
    fn default() -> Self {
        TunnelOptions {
            openvpn: openvpn::TunnelOptions::default(),
            wireguard: wireguard::TunnelOptions {
                mtu: None,
                automatic_rotation: None,
            },
            generic: GenericTunnelOptions { enable_ipv6: false },
        }
    }
}
