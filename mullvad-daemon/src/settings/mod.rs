#[cfg(not(target_os = "android"))]
use futures::TryFutureExt;
use mullvad_types::{
    relay_constraints::{RelayConstraints, RelaySettings, WireguardConstraints},
    settings::{DnsState, Settings},
};
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

pub mod patch;

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

    #[error(display = "Failed to apply settings update")]
    UpdateFailed(Box<dyn std::error::Error + Send + Sync>),
}

/// Converts an [Error] to a management interface status
#[cfg(not(target_os = "android"))]
impl From<Error> for mullvad_management_interface::Status {
    fn from(error: Error) -> mullvad_management_interface::Status {
        use mullvad_management_interface::{Code, Status};

        match error {
            Error::DeleteError(..) | Error::WriteError(..) | Error::ReadError(..) => {
                Status::new(Code::FailedPrecondition, error.to_string())
            }
            Error::SerializeError(..) | Error::ParseError(..) | Error::UpdateFailed(..) => {
                Status::new(Code::Internal, error.to_string())
            }
        }
    }
}

pub struct SettingsPersister {
    settings: Settings,
    path: PathBuf,
    #[allow(clippy::type_complexity)]
    on_change_listeners: Vec<Box<dyn Fn(&Settings)>>,
}

pub type MadeChanges = bool;

impl SettingsPersister {
    /// Loads user settings from file. If it fails, it returns the defaults.
    pub async fn load(settings_dir: &Path) -> Self {
        let path = settings_dir.join(SETTINGS_FILE);
        let LoadSettingsResult {
            mut settings,
            mut should_save,
        } = Self::load_inner(|| Self::load_from_file(&path)).await;

        // Force IPv6 to be enabled on Android
        if cfg!(target_os = "android") {
            should_save |= !settings.tunnel_options.generic.enable_ipv6;
            settings.tunnel_options.generic.enable_ipv6 = true;
        }
        if crate::version::is_beta_version() {
            should_save |= !settings.show_beta_releases;
            settings.show_beta_releases = true;
        }

        let mut persister = SettingsPersister {
            settings,
            path,
            on_change_listeners: vec![],
        };

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

    /// Loads user settings, returning default settings if it should fail.
    ///
    /// `load_settings` allows the caller to decide how to load [`Settings`]
    /// from an bitrary resource.
    ///
    /// `load_inner` will always succeed, even in the presence of IO operations.
    /// Errors are handled gracefully by returning the default [`Settings`] if
    /// necessary.
    async fn load_inner<F, R>(load_settings: F) -> LoadSettingsResult
    where
        F: FnOnce() -> R,
        R: std::future::Future<Output = Result<Settings, Error>>,
    {
        match load_settings().await {
            Ok(settings) => LoadSettingsResult {
                settings,
                should_save: false,
            },
            Err(Error::ReadError(_, err)) if err.kind() == io::ErrorKind::NotFound => {
                log::info!("No settings were found. Using defaults.");
                LoadSettingsResult {
                    settings: Self::default_settings(),
                    should_save: true,
                }
            }
            Err(error) => {
                log::warn!(
                    "{}",
                    error.display_chain_with_msg("Failed to load settings. Using defaults.")
                );
                let mut settings = Self::default_settings();

                // Protect the user by blocking the internet by default. Previous settings may
                // not have caused the daemon to enter the non-blocking disconnected state.
                settings.block_when_disconnected = true;

                LoadSettingsResult {
                    settings,
                    should_save: true,
                }
            }
        }
    }

    async fn load_from_file<P>(path: P) -> Result<Settings, Error>
    where
        P: AsRef<Path> + Clone,
    {
        let display = path.clone();
        log::info!("Loading settings from {}", display.as_ref().display());
        let settings_bytes = fs::read(path)
            .await
            .map_err(|error| Error::ReadError(display.as_ref().display().to_string(), error))?;
        let settings = Self::load_from_bytes(&settings_bytes)?;
        Ok(settings)
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
            .await?;

        self.notify_listeners();

        Ok(())
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

    /// Edit the settings in a closure and write the changes to disk.
    ///
    /// # On success
    ///
    /// Returns a boolean indicating whether any settings were changed.
    ///
    /// # On failure
    ///
    /// If the settings could not be written to disk, all changes are rolled
    /// back, and an error is returned.
    ///
    /// # Note
    ///
    /// If no settings were changed, no I/O will be performed.
    pub async fn update(
        &mut self,
        update_fn: impl FnOnce(&mut Settings),
    ) -> Result<MadeChanges, Error> {
        self.try_update(|settings| -> Result<(), Error> {
            update_fn(settings);
            Ok(())
        })
        .await
    }

    /// Edit the settings in a closure, and write the changes to disk.
    ///
    /// # On success
    ///
    /// Returns a boolean indicating whether any settings were changed.
    ///
    /// # On failure
    ///
    /// `try_update` may fail in two scenarios
    ///
    /// ## The settings could not be written to disk
    ///
    /// In this case, all changes are rolled back and an error is returned.
    ///
    /// ## `update_fn` failed
    ///
    /// If `update_fn` were to fail the error will be propagated through the
    /// [`Error::UpdateFailed`] error variant. Since the error will be boxed, it
    /// has to be downcasted at runtime using [`Box::downcast`] in case you want
    /// to inspect the error closer.
    ///
    /// ```ignore
    /// #[derive(Debug, err_derive::Error)]
    /// pub enum MyError {
    ///   #[error(display = "Failed for this reason: {:?}", _0)]
    ///   Failed(String),
    /// }
    ///
    /// let settings = Settings::default_settings();
    /// let err = settings.try_update(|settings| {
    ///   // Perform some update on the settings
    ///   settings.allow_lan = !settings.allow_lan;
    ///   // Fail the update procedure due to some error
    ///   Err(MyError::Failed("No particular reason".to_string()))
    /// });
    ///
    /// matches!(err, Error::UpdateFailed(_)) ;
    /// assert_eq!(settings, Settings::default_settings())
    /// ```
    pub async fn try_update<E>(
        &mut self,
        update_fn: impl FnOnce(&mut Settings) -> Result<(), E>,
    ) -> Result<MadeChanges, Error>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut new_settings = self.settings.clone();

        update_fn(&mut new_settings)
            .map_err(Box::from)
            .map_err(Error::UpdateFailed)?;

        if self.settings == new_settings {
            return Ok(false);
        }

        Self::save_inner(&self.path, &new_settings).await?;
        self.settings = new_settings;

        self.notify_listeners();

        Ok(true)
    }

    /// Return a compact summary of important settings
    pub fn summary(&self) -> SettingsSummary<'_> {
        SettingsSummary {
            settings: &self.settings,
        }
    }

    pub fn register_change_listener(&mut self, change_listener: impl Fn(&Settings) + 'static) {
        self.on_change_listeners.push(Box::new(change_listener));
    }

    fn notify_listeners(&self) {
        for listener in &self.on_change_listeners {
            listener(&self.settings);
        }
    }
}

struct LoadSettingsResult {
    settings: Settings,
    should_save: bool,
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

        let multihop = match relay_settings {
            RelaySettings::Normal(RelayConstraints {
                wireguard_constraints,
                ..
            }) => wireguard_constraints.multihop(),
            _ => false,
        };

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

        write!(f, ", dns: ")?;

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
                if default_options.block_social_media {
                    content.push("social media");
                }
                if content.is_empty() {
                    content.push("default");
                }
                write!(f, "{}", content.join(" "))?;
            }
            DnsState::Custom => {
                // NOTE: Technically inaccurate, as the gateway IP is a local IP but isn't treated
                // as one.
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
                    (true, true) => f.write_str("custom, public, local")?,
                    (true, false) => f.write_str("custom, public")?,
                    (false, false) => f.write_str("custom, no addrs")?,
                    (false, true) => f.write_str("custom, local")?,
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
    use super::*;
    use mullvad_types::settings::SettingsVersion;

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
                      "location": {
                        "country": "gb"
                      }
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
                "bridge_type": "normal",
                "normal": {
                  "location": "any"
                },
                "custom": {
                  "socks5_local": {
                    "local_port": 1080,
                    "remote_endpoint": {
                      "address": "1.3.3.7:22",
                      "protocol": "tcp"
                    }
                  }
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
              "settings_version": 8,
              "show_beta_releases": false,
              "custom_lists": {
                "custom_lists": []
              }
        }"#;

        let _ = SettingsPersister::load_from_bytes(settings).unwrap();
    }

    /// The [`SettingsPersister`] should always succeed when deserializing a
    /// [`Settings`] object from disk. However, there is a distinction between
    /// different error cases.
    ///
    /// If the settings file is missing, it could be because the user starts the
    /// app for the first time. As such, we should simply save the default
    /// [`Settings`] to disk.
    #[tokio::test]
    async fn test_deserialize_missing_settings() {
        let LoadSettingsResult {
            should_save,
            settings,
        } = SettingsPersister::load_inner(|| async {
            Err(Error::ReadError(
                "Settings are missing".to_string(),
                io::ErrorKind::NotFound.into(),
            ))
        })
        .await;

        assert!(
            should_save,
            "Settings should be saved to disk if they didn't exist previously"
        );

        assert!(
            !settings.block_when_disconnected,
            "The daemon should not block the internet if settings are missing"
        );
    }

    /// The [`SettingsPersister`] should always succeed when deserializing a
    /// [`Settings`] object from disk. However, there is a distinction between
    /// different error cases.
    ///
    /// If the settings file is corrupt, we can assume that the user has started
    /// the app previously, but we can't know what settings the user have
    /// changed. In this case, we should safeguard against leaks by locking down
    /// the network before the user initiates a connection attempt or change
    /// these settings.
    #[tokio::test]
    async fn test_deserialize_invalid_settings() {
        let LoadSettingsResult {
            should_save,
            settings,
        } = SettingsPersister::load_inner(|| async {
            SettingsPersister::load_from_bytes(b"Not a valid settings file")
        })
        .await;

        assert!(
            should_save,
            "Settings should be saved to disk if they have become corrupt"
        );

        assert!(
            settings.block_when_disconnected,
            "The daemon should block the internet if settings are corrupt"
        );
    }
}
