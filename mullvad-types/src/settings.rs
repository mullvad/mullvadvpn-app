use crate::relay_constraints::{
    Constraint, LocationConstraint, RelayConstraints, RelaySettings, RelaySettingsUpdate,
};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io, path::PathBuf};
use talpid_types::net::{OpenVpnProxySettings, OpenVpnProxySettingsValidation, TunnelOptions};

error_chain! {
    errors {
        DirectoryError {
            description("Unable to create settings directory for program")
        }
        ReadError(path: PathBuf) {
            description("Unable to read settings file")
            display("Unable to read settings from {}", path.display())
        }
        WriteError(path: PathBuf) {
            description("Unable to write settings file")
            display("Unable to write settings to {}", path.display())
        }
        ParseError {
            description("Malformed settings")
        }
        InvalidProxyData(reason: String) {
            description("Invalid proxy configuration was rejected")
            display("Invalid proxy configuration was rejected: {}", reason)
        }
    }
}

static SETTINGS_FILE: &str = "settings.json";


/// Mullvad daemon settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    account_token: Option<String>,
    relay_settings: RelaySettings,
    /// If the daemon should allow communication with private (LAN) networks.
    allow_lan: bool,
    /// Extra level of kill switch. When this setting is on, the disconnected state will block
    /// the firewall to not allow any traffic in or out.
    block_when_disconnected: bool,
    /// If the daemon should connect the VPN tunnel directly on start or not.
    auto_connect: bool,
    /// Options that should be applied to tunnels of a specific type regardless of where the relays
    /// might be located.
    tunnel_options: TunnelOptions,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            account_token: None,
            relay_settings: RelaySettings::Normal(RelayConstraints {
                location: Constraint::Only(LocationConstraint::Country("se".to_owned())),
                tunnel: Constraint::Any,
            }),
            allow_lan: false,
            block_when_disconnected: false,
            auto_connect: false,
            tunnel_options: TunnelOptions::default(),
        }
    }
}

impl Settings {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load() -> Result<Settings> {
        let settings_path = Self::get_settings_path()?;
        match File::open(&settings_path) {
            Ok(file) => {
                info!("Loading settings from {}", settings_path.display());
                Self::read_settings(&mut io::BufReader::new(file))
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!(
                    "No settings file at {}, using defaults",
                    settings_path.display()
                );
                Ok(Settings::default())
            }
            Err(e) => Err(e).chain_err(|| ErrorKind::ReadError(settings_path)),
        }
    }

    /// Serializes the settings and saves them to the file it was loaded from.
    fn save(&self) -> Result<()> {
        let path = Self::get_settings_path()?;

        debug!("Writing settings to {}", path.display());
        let mut file = File::create(&path).chain_err(|| ErrorKind::WriteError(path.clone()))?;

        serde_json::to_writer_pretty(&mut file, self)
            .chain_err(|| ErrorKind::WriteError(path.clone()))?;
        file.sync_all().chain_err(|| ErrorKind::WriteError(path))
    }

    fn get_settings_path() -> Result<PathBuf> {
        let dir = ::mullvad_paths::settings_dir().chain_err(|| ErrorKind::DirectoryError)?;
        Ok(dir.join(SETTINGS_FILE))
    }

    fn read_settings<T: io::Read>(file: &mut T) -> Result<Settings> {
        serde_json::from_reader(file).chain_err(|| ErrorKind::ParseError)
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
        let new_settings = self.relay_settings.merge(update);
        if self.relay_settings != new_settings {
            debug!(
                "changing relay settings from {} to {}",
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

    pub fn set_openvpn_proxy(&mut self, proxy: Option<OpenVpnProxySettings>) -> Result<bool> {
        if let Some(ref settings) = proxy {
            if let Err(validation_error) = OpenVpnProxySettingsValidation::validate(settings) {
                bail!(ErrorKind::InvalidProxyData(validation_error));
            }
        }

        if self.tunnel_options.openvpn.proxy != proxy {
            self.tunnel_options.openvpn.proxy = proxy;
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

    pub fn get_tunnel_options(&self) -> &TunnelOptions {
        &self.tunnel_options
    }
}
