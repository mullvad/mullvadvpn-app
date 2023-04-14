#[cfg(not(target_os = "android"))]
use futures::TryFutureExt;
use mullvad_types::{
    relay_constraints::{RelayConstraints, RelaySettings, WireguardConstraints},
    settings::{DnsState, Settings},
};
use rand::Rng;
use std::{
    fmt::{self, Display},
    ops::Deref,
    path::{Path, PathBuf},
};
use talpid_core::firewall::is_local_address;
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

    async fn save(&mut self) -> Result<(), Error> {
        Self::save_inner(&self.path, &self.settings).await
    }

    /// Serializes the settings and saves them to the given file.
    async fn save_inner(path: &Path, settings: &Settings) -> Result<(), Error> {
        log::debug!("Writing settings to {}", path.display());

        let buffer = serde_json::to_string_pretty(settings).map_err(Error::SerializeError)?;
        let mut file = mullvad_fs::AtomicFile::new(path)
            .await
            .map_err(|e| Error::WriteError(path.display().to_string(), e))?;
        file.write_all(&buffer.into_bytes())
            .await
            .map_err(|e| Error::WriteError(path.display().to_string(), e))?;
        file.finalize()
            .await
            .map_err(|e| Error::WriteError(path.display().to_string(), e))?;

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
        let mut new_settings = self.settings.clone();

        update_fn(&mut new_settings);

        if self.settings == new_settings {
            return Ok(false);
        }

        Self::save_inner(&self.path, &new_settings).await?;
        self.settings = new_settings;
        Ok(true)
    }

    /// Return a compact summary of important settings
    pub fn summary(&self) -> SettingsSummary<'_> {
        SettingsSummary {
            settings: &self.settings,
        }
    }
}

impl Deref for SettingsPersister {
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

/// A compact summary of important settings
pub struct SettingsSummary<'a> {
    settings: &'a Settings,
}

impl<'a> Display for SettingsSummary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bool_to_label = |state| {
            if state {
                "on"
            } else {
                "off"
            }
        };

        let relay_settings = self.settings.get_relay_settings();

        write!(f, "openvpn mssfix: ")?;
        Self::fmt_option(f, self.settings.tunnel_options.openvpn.mssfix)?;
        write!(f, ", wg mtu: ")?;
        Self::fmt_option(f, self.settings.tunnel_options.wireguard.mtu)?;

        if let RelaySettings::Normal(RelayConstraints {
            wireguard_constraints: WireguardConstraints { ip_version, .. },
            ..
        }) = relay_settings
        {
            write!(f, ", wg ip version: {ip_version}")?;
        }

        let multihop = matches!(
            relay_settings,
            RelaySettings::Normal(RelayConstraints {
                wireguard_constraints: WireguardConstraints {
                    use_multihop: true,
                    ..
                },
                ..
            })
        );

        write!(
            f,
            ", multihop: {}, ipv6 (tun): {}, lan: {}, pq: {}, obfs: {}",
            bool_to_label(multihop),
            bool_to_label(self.settings.tunnel_options.generic.enable_ipv6),
            bool_to_label(self.settings.allow_lan),
            self.settings.tunnel_options.wireguard.quantum_resistant,
            self.settings.obfuscation_settings.selected_obfuscation,
        )?;

        // Print DNS options

        match self.settings.tunnel_options.dns_options.state {
            DnsState::Default => {
                let mut content = vec![];
                let default_options = &self.settings.tunnel_options.dns_options.default_options;

                if default_options.block_ads {
                    content.push("ads");
                }
                if default_options.block_trackers {
                    content.push("trackers");
                }
                if default_options.block_malware {
                    content.push("malware");
                }
                if default_options.block_adult_content {
                    content.push("adult");
                }
                if default_options.block_gambling {
                    content.push("gambling");
                }
                if content.is_empty() {
                    content.push("default");
                }

                write!(f, ", dns: {}", content.join(" "))?;
            }
            DnsState::Custom => {
                // NOTE: Technically inaccurate, as the gateway IP is a local IP but isn't treated as one.
                let contains_local = self
                    .settings
                    .tunnel_options
                    .dns_options
                    .custom_options
                    .addresses
                    .iter()
                    .any(is_local_address);
                let contains_public = self
                    .settings
                    .tunnel_options
                    .dns_options
                    .custom_options
                    .addresses
                    .iter()
                    .any(|addr| !is_local_address(addr));

                match (contains_public, contains_local) {
                    (true, true) => f.write_str(", dns: custom, public, local")?,
                    (true, false) => f.write_str(", dns: custom, public")?,
                    (false, false) => f.write_str(", dns: custom, no addrs")?,
                    (false, true) => f.write_str(", dns: custom, local")?,
                }
            }
        }
        Ok(())
    }
}

impl<'a> SettingsSummary<'a> {
    fn fmt_option<T: Display>(f: &mut fmt::Formatter<'_>, val: Option<T>) -> fmt::Result {
        if let Some(inner) = &val {
            inner.fmt(f)
        } else {
            f.write_str("unset")
        }
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
