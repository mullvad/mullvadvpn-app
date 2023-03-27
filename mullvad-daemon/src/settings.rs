#[cfg(not(target_os = "android"))]
use futures::TryFutureExt;
use mullvad_types::settings::Settings;
use rand::Rng;
#[cfg(target_os = "windows")]
use std::collections::HashSet;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

const SETTINGS_FILE: &str = "settings.json";

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Unable to read settings file {}", _0)]
    ReadError(String, #[error(source)] io::Error),

    #[error(display = "Unable to parse settings file")]
    ParseError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to remove settings file {}", _0)]
    #[cfg(not(target_os = "android"))]
    DeleteError(String, #[error(source)] io::Error),

    #[error(display = "Unable to serialize settings to JSON")]
    SerializeError(#[error(source)] serde_json::Error),

    #[error(display = "Unable to write settings to {}", _0)]
    WriteError(String, #[error(source)] io::Error),
}

#[derive(Debug)]
pub struct SettingsPersister {
    settings: Settings,
    path: PathBuf,
}

pub type MadeChanges = bool;

impl SettingsPersister {
    /// Loads user settings from file. If it fails, it returns the defaults.
    pub async fn load(settings_dir: &Path) -> Self {
        let path = settings_dir.join(SETTINGS_FILE);
        let (mut settings, mut should_save) = match Self::load_from_file(&path).await {
            Ok(value) => value,
            Err(error) => {
                log::warn!(
                    "{}",
                    error.display_chain_with_msg("Failed to load settings. Using defaults.")
                );
                let mut settings = Self::default_settings();

                // Protect the user by blocking the internet by default. Previous settings may
                // not have caused the daemon to enter the non-blocking disconnected state.
                settings.block_when_disconnected = true;

                (settings, true)
            }
        };

        // If the settings file did not contain a wg_migration_rand_num then it will be initialized
        // to -1.0 by serde. This block ensures that this value is correctly intitialzed to a
        // percentage.
        if settings.wg_migration_rand_num < 0.0 || settings.wg_migration_rand_num > 1.0 {
            let mut rng = rand::thread_rng();
            settings.wg_migration_rand_num = rng.gen_range(0.0..=1.0);
            should_save |= true
        }

        // Force IPv6 to be enabled on Android
        if cfg!(target_os = "android") {
            should_save |= !settings.tunnel_options.generic.enable_ipv6;
            settings.tunnel_options.generic.enable_ipv6 = true;
        }
        if crate::version::is_beta_version() {
            should_save |= !settings.show_beta_releases;
            settings.show_beta_releases = true;
        }

        let mut persister = SettingsPersister { settings, path };

        if should_save {
            if let Err(error) = persister.save().await {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to save updated settings")
                );
            }
        }

        persister
    }

    async fn load_from_file(path: &Path) -> Result<(Settings, bool), Error> {
        log::info!("Loading settings from {}", path.display());

        let settings_bytes = match fs::read(path).await {
            Ok(bytes) => bytes,
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    log::info!("No settings were found. Using defaults.");
                    return Ok((Self::default_settings(), true));
                } else {
                    return Err(Error::ReadError(path.display().to_string(), error));
                }
            }
        };
        Ok((Self::load_from_bytes(&settings_bytes)?, false))
    }

    fn load_from_bytes(bytes: &[u8]) -> Result<Settings, Error> {
        serde_json::from_slice(bytes).map_err(Error::ParseError)
    }

    /// Serializes the settings and saves them to the file it was loaded from.
    async fn save(&mut self) -> Result<(), Error> {
        log::debug!("Writing settings to {}", self.path.display());

        let buffer = serde_json::to_string_pretty(&self.settings).map_err(Error::SerializeError)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))?;
        file.write_all(&buffer.into_bytes())
            .await
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))?;
        file.sync_all()
            .await
            .map_err(|e| Error::WriteError(self.path.display().to_string(), e))?;

        Ok(())
    }

    /// Resets default settings
    #[cfg(not(target_os = "android"))]
    pub async fn reset(&mut self) -> Result<(), Error> {
        self.settings = Self::default_settings();
        let path = self.path.clone();
        self.save()
            .or_else(|e| async move {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Unable to save default settings")
                );
                log::info!("Will attempt to remove settings file");
                fs::remove_file(&path)
                    .map_err(|e| Error::DeleteError(path.display().to_string(), e))
                    .await
            })
            .await
    }

    pub fn to_settings(&self) -> Settings {
        self.settings.clone()
    }

    /// Modifies `Settings::default()` somewhat, e.g. depending on whether a beta version
    /// is being run or not.
    fn default_settings() -> Settings {
        let mut settings = Settings::default();

        if crate::version::is_beta_version() {
            settings.show_beta_releases = true;
        }
        settings
    }

    /// Edit the settings in a closure, and write the changes, if any, to disk.
    ///
    /// On success, the function returns a boolean indicating whether any settings were changed.
    /// If the settings could not be written to disk, all changes are rolled back, and an error is
    /// returned.
    pub async fn update(
        &mut self,
        update_fn: impl FnOnce(&mut Settings),
    ) -> Result<MadeChanges, Error> {
        let backup = self.settings.clone();

        update_fn(&mut self.settings);

        if self.settings == backup {
            return Ok(false);
        }

        match self.save().await {
            Ok(()) => Ok(true),
            Err(error) => {
                // restore state
                self.settings = backup;
                Err(error)
            }
        }
    }
}

impl Deref for SettingsPersister {
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

#[cfg(test)]
mod test {
    use super::SettingsPersister;
    use mullvad_types::settings::SettingsVersion;
    use serde_json;

    #[test]
    #[should_panic]
    fn test_deserialization_failure_version_too_small() {
        let _version: SettingsVersion = serde_json::from_str("1").expect("Version too small");
    }

    #[test]
    #[should_panic]
    fn test_deserialization_failure_version_too_big() {
        let _version: SettingsVersion = serde_json::from_str("1000").expect("Version too big");
    }

    #[test]
    fn test_deserialization_success() {
        let _version: SettingsVersion =
            serde_json::from_str("2").expect("Failed to deserialize valid version");
    }

    #[test]
    fn test_serialization_success() {
        let version = SettingsVersion::V2;
        let s = serde_json::to_string(&version).expect("Failed to serialize");
        assert_eq!(s, "2");
    }

    #[test]
    fn test_deserialization() {
        let settings = br#"{
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
                  "rotation_interval": null
                },
                "generic": {
                  "enable_ipv6": true
                }
              },
              "settings_version": 5,
              "show_beta_releases": false
        }"#;

        let _ = SettingsPersister::load_from_bytes(settings).unwrap();
    }
}
