extern crate serde_json;

use app_dirs;

use mullvad_types::relay_constraints::{Constraint, LocationConstraint, RelayConstraints,
                                       RelaySettings, RelaySettingsUpdate};
use talpid_types::net::TunnelOptions;

use std::fs::File;
use std::io;
use std::path::PathBuf;

error_chain! {
    errors {
        DirectoryError {
            description("Unable to create settings directory for program")
        }
        ReadError(path: PathBuf) {
            description("Unable to read settings file")
            display("Unable to read settings from {}", path.to_string_lossy())
        }
        WriteError(path: PathBuf) {
            description("Unable to write settings file")
            display("Unable to write settings to {}", path.to_string_lossy())
        }
        ParseError {
            description("Malformed settings")
        }
    }
}

static SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    account_token: Option<String>,
    relay_settings: RelaySettings,
    /// If the app should allow communication with private (LAN) networks.
    allow_lan: bool,
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
                info!("Loading settings from {}", settings_path.to_string_lossy());
                Self::read_settings(&mut io::BufReader::new(file))
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!(
                    "No settings file at {}, using defaults",
                    settings_path.to_string_lossy()
                );
                Ok(Settings::default())
            }
            Err(e) => Err(e).chain_err(|| ErrorKind::ReadError(settings_path)),
        }
    }

    /// Serializes the settings and saves them to the file it was loaded from.
    fn save(&self) -> Result<()> {
        let path = Self::get_settings_path()?;

        debug!("Writing settings to {}", path.to_string_lossy());
        let file = File::create(&path).chain_err(|| ErrorKind::WriteError(path.clone()))?;

        serde_json::to_writer_pretty(file, self).chain_err(|| ErrorKind::WriteError(path))
    }

    fn get_settings_path() -> Result<PathBuf> {
        let dir = app_dirs::app_root(app_dirs::AppDataType::UserConfig, &::APP_INFO)
            .chain_err(|| ErrorKind::DirectoryError)?;
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
                "changing relay settings from {:?} to {:?}",
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

    pub fn set_openvpn_mssfix(&mut self, openvpn_mssfix: Option<u16>) -> Result<bool> {
        if self.tunnel_options.openvpn.mssfix != openvpn_mssfix {
            self.tunnel_options.openvpn.mssfix = openvpn_mssfix;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    pub fn get_tunnel_options(&self) -> &TunnelOptions {
        &self.tunnel_options
    }
}
