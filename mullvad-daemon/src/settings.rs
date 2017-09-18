extern crate app_dirs;
extern crate toml;

use self::app_dirs::{AppDataType, AppInfo};

use mullvad_types::relay_endpoint::RelayEndpoint;

use std::fs::File;
use std::io::{self, Read, Write};
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

static SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    account_token: Option<String>,
    custom_relay: Option<RelayEndpoint>,
}

const DEFAULT_SETTINGS: Settings = Settings {
    account_token: None,
    custom_relay: None,
};

impl Settings {
    /// Loads user settings from file. If no file is present it returns the defaults.
    pub fn load() -> Result<Settings> {
        let settings_path = Self::get_settings_path()?;
        match File::open(&settings_path) {
            Ok(mut file) => {
                info!("Loading settings from {}", settings_path.to_string_lossy());
                Self::read_settings(&mut file, settings_path)
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
        let settings_path = Self::get_settings_path()?;
        let data = toml::to_string(self).chain_err(|| ErrorKind::ParseError)?;

        debug!("Writing settings to {}", settings_path.to_string_lossy());
        let mut file = File::create(&settings_path)
            .chain_err(|| ErrorKind::WriteError(settings_path.clone()))?;
        file.write_all(data.as_bytes())
            .chain_err(|| ErrorKind::WriteError(settings_path))?;
        Ok(())
    }

    fn get_settings_path() -> Result<PathBuf> {
        let dir = app_dirs::app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| ErrorKind::DirectoryError)?;
        Ok(dir.join(SETTINGS_FILE))
    }

    fn read_settings(file: &mut File, path: PathBuf) -> Result<Settings> {
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .chain_err(|| ErrorKind::ReadError(path))?;
        toml::from_slice(&data).chain_err(|| ErrorKind::ParseError)
    }

    pub fn get_account_token(&self) -> Option<String> {
        self.account_token.clone()
    }

    /// Changes account number to the one given. Also saves the new settings to disk.
    /// The boolean in the Result indicates if the account token changed or not
    pub fn set_account_token(&mut self, account_token: Option<String>) -> Result<bool> {
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

    pub fn get_custom_relay(&self) -> Option<RelayEndpoint> {
        self.custom_relay.clone()
    }

    pub fn set_custom_relay(&mut self, relay_endpoint: Option<RelayEndpoint>) -> Result<bool> {
        if self.custom_relay != relay_endpoint {
            match &relay_endpoint {
                &Some(ref data) => info!("Setting a custom relay: {}", data),
                &None => info!("Removing the custom relay"),
            }

            self.custom_relay = relay_endpoint;

            self.save().map(|_| true)
        } else {
            Ok(false)
        }
    }
}
