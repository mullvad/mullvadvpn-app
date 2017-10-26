extern crate app_dirs;
extern crate serde_json;

use self::app_dirs::{AppDataType, AppInfo};

use talpid_types::net::TransportProtocol;

use mullvad_types::relay_constraints::{OpenVpnConstraints, Port, RelayConstraints,
                                       RelayConstraintsUpdate, TunnelConstraints};
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

static APP_INFO: AppInfo = AppInfo {
    name: ::CRATE_NAME,
    author: "Mullvad",
};

static SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    account_token: Option<String>,
    relay_constraints: RelayConstraints,
}

impl Default for Settings {
    fn default() -> Self {
        DEFAULT_SETTINGS
    }
}

const DEFAULT_SETTINGS: Settings = Settings {
    account_token: None,
    relay_constraints: RelayConstraints {
        host: None,
        tunnel: TunnelConstraints::OpenVpn(OpenVpnConstraints {
            port: Port::Any,
            protocol: TransportProtocol::Udp,
        }),
    },
};

impl Settings {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load() -> Result<Settings> {
        let settings_path = Self::get_settings_path()?;
        match File::open(&settings_path) {
            Ok(mut file) => {
                info!("Loading settings from {}", settings_path.to_string_lossy());
                Self::read_settings(&mut file)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!(
                    "No settings file at {}, using defaults",
                    settings_path.to_string_lossy()
                );
                Ok(DEFAULT_SETTINGS)
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
        let dir = app_dirs::app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| ErrorKind::DirectoryError)?;
        Ok(dir.join(SETTINGS_FILE))
    }

    fn read_settings(file: &mut File) -> Result<Settings> {
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
            info!(
                "Changing account token from {} to {}",
                Self::format_account_token(&self.account_token),
                Self::format_account_token(&account_token),
            );
            self.account_token = account_token;
            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn format_account_token(account_token: &Option<String>) -> String {
        match *account_token {
            Some(ref account_token) => format!("\"{}\"", account_token),
            None => "[none]".to_owned(),
        }
    }

    pub fn get_relay_constraints(&self) -> RelayConstraints {
        self.relay_constraints.clone()
    }

    pub fn update_relay_constraints(&mut self, update: RelayConstraintsUpdate) -> Result<bool> {
        let new_constraints = self.relay_constraints.update(update);

        if new_constraints {
            debug!(
                "changed relay constraints from {:?} to {:?}",
                self.relay_constraints,
                new_constraints
            );

            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }
}
