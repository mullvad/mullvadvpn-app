use std::io;
use std::path::{Path, PathBuf};

use futures::channel::oneshot;
use talpid_core::{mpsc::Sender, tunnel_state_machine::TunnelCommand};
use talpid_error::ErrorExt;
use talpid_types::{
    Stats,
    net::wireguard::{CustomVpnConfig, CustomVpnConfigUpdate, CustomVpnSettings},
};
use tokio::io::AsyncWriteExt;

use crate::{Daemon, InternalDaemonEvent};

pub(crate) const CUSTOM_VPN_SETTINGS_FILE: &str = "custom-vpn.json";

/// Commands related to the custom VPN.
pub enum CustomVpnCommand {
    /// Set custom VPN configuration (None clears all configs)
    SetConfig(oneshot::Sender<String>, Option<CustomVpnConfigUpdate>),
    /// Get custom VPN settings
    GetSettings(oneshot::Sender<CustomVpnSettings>),
    /// Get custom VPN stats
    GetStats(oneshot::Sender<Stats>),
}

/// Act on a custom VPN command.
pub(crate) async fn handle_command(daemon: &mut Daemon, command: CustomVpnCommand) {
    match command {
        CustomVpnCommand::SetConfig(tx, config) => {
            set_custom_vpn_config(daemon, tx, config).await;
        }
        CustomVpnCommand::GetSettings(tx) => get_custom_vpn_settings(daemon, tx),
        CustomVpnCommand::GetStats(tx) => get_custom_vpn_stats(daemon, tx),
    }
}

/// Clear the custom VPN store and stop using it for new tunnels.
pub(crate) async fn reset(daemon: &mut Daemon) {
    match daemon.custom_vpn.set(CustomVpnSettings::default()).await {
        Ok(()) => {
            let _ = daemon
                .tx
                .send(InternalDaemonEvent::CustomVpnSettingsChanged);
            daemon.parameters_generator.set_custom_vpn(None).await;
        }
        Err(error) => log::error!(
            "{}",
            error.display_chain_with_msg("Failed to reset custom VPN settings")
        ),
    }
}

async fn set_custom_vpn_config(
    daemon: &mut Daemon,
    tx: oneshot::Sender<String>,
    config: Option<CustomVpnConfigUpdate>,
) {
    let new_settings = match config {
        // Clear all configs
        None => CustomVpnSettings::default(),
        Some(update) => {
            // Only a single config is currently managed
            let config = match update.resolve(daemon.custom_vpn.settings().configs.first()) {
                Ok(config) => config,
                Err(err) => {
                    log::error!("Failed to resolve custom VPN config: {err}");
                    Daemon::oneshot_send(tx, err, "set_custom_vpn_config");
                    return;
                }
            };
            CustomVpnSettings {
                configs: vec![config],
            }
        }
    };

    if let Err(err) = daemon.custom_vpn.set(new_settings).await {
        log::error!(
            "{}",
            err.display_chain_with_msg("Failed to set custom VPN config")
        );
        Daemon::oneshot_send(tx, err.to_string(), "set_custom_vpn_config");
        return;
    }

    let effective = daemon.custom_vpn.settings().active().cloned();
    daemon.parameters_generator.set_custom_vpn(effective).await;
    Daemon::oneshot_send(tx, String::new(), "set_custom_vpn_config");
    daemon.reconnect_tunnel();

    // The custom VPN is not part of `Settings`, so changing it does not
    // trigger the settings listeners. Update the feature indicators and
    // notify listening clients of the new custom VPN settings.
    let _ = daemon
        .tx
        .send(InternalDaemonEvent::CustomVpnSettingsChanged);
}

fn get_custom_vpn_settings(daemon: &Daemon, tx: oneshot::Sender<CustomVpnSettings>) {
    Daemon::oneshot_send(
        tx,
        daemon.custom_vpn.to_settings(),
        "get_custom_vpn_settings",
    );
}

fn get_custom_vpn_stats(daemon: &Daemon, tx: oneshot::Sender<Stats>) {
    daemon.send_tunnel_command(TunnelCommand::CustomVpnStats(tx));
}

/// Loads, stores and updates the custom VPN settings. They contain a
/// private key, so the file is written with the same permissions as
/// `device.json`.
pub struct CustomVpnSettingsPersister {
    settings: CustomVpnSettings,
    path: PathBuf,
}

impl CustomVpnSettingsPersister {
    /// Load the custom VPN settings from `<settings_dir>/custom-vpn.json`.
    /// A missing or unparsable file results in the default settings, and is
    /// left untouched.
    pub async fn load(settings_dir: &Path) -> Self {
        let path = settings_dir.join(CUSTOM_VPN_SETTINGS_FILE);
        let settings = Self::load_from_file(&path).await;
        Self { settings, path }
    }

    async fn load_from_file(path: &Path) -> CustomVpnSettings {
        log::info!("Loading custom VPN settings from {}", path.display());
        match tokio::fs::read(path).await {
            Ok(bytes) => match serde_json::from_slice::<CustomVpnSettings>(&bytes) {
                Ok(settings) => settings,
                Err(error) => {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(&format!(
                            "Failed to parse {}. Using default settings. The file is left \
                             untouched on disk",
                            path.display()
                        ))
                    );
                    CustomVpnSettings::default()
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                log::info!("No custom VPN settings file found. Using default settings.");
                CustomVpnSettings::default()
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(&format!(
                        "Failed to read {}. Using default settings.",
                        path.display()
                    ))
                );
                CustomVpnSettings::default()
            }
        }
    }

    pub fn settings(&self) -> &CustomVpnSettings {
        &self.settings
    }

    /// Whether any stored config is enabled.
    pub fn active(&self) -> bool {
        self.settings.active().is_some()
    }

    /// The enabled config, if any.
    pub fn active_config(&self) -> Option<CustomVpnConfig> {
        self.settings.active().cloned()
    }

    pub fn to_settings(&self) -> CustomVpnSettings {
        self.settings.clone()
    }

    /// Replace the custom VPN settings, writing them to disk before updating
    /// the in-memory state.
    pub async fn set(&mut self, settings: CustomVpnSettings) -> Result<(), io::Error> {
        let content = serde_json::to_string_pretty(&settings)?;
        #[cfg(unix)]
        let mut file = mullvad_fs::AtomicFile::new_with_mode(&self.path, 0o600).await?;
        // On Windows the temp file inherits the permissions of the settings
        // directory.
        #[cfg(not(unix))]
        let mut file = mullvad_fs::AtomicFile::new(&self.path).await?;
        file.write_all(content.as_bytes()).await?;
        file.finalize().await?;
        self.settings = settings;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use talpid_types::net::wireguard::{
        CustomVpnConfigUpdate, CustomVpnPeerConfig, CustomVpnTunnelConfig, CustomVpnTunnelUpdate,
        PrivateKey, PublicKey,
    };
    use vec1::vec1;

    fn sample_config() -> CustomVpnConfig {
        CustomVpnConfig {
            enabled: true,
            name: Some("sample".to_string()),
            tunnel: CustomVpnTunnelConfig {
                private_key: PrivateKey::new_from_random(),
                ips: vec!["10.0.0.2".parse().unwrap()],
            },
            peers: vec1![CustomVpnPeerConfig {
                public_key: PublicKey::from_base64("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBE=")
                    .unwrap(),
                allowed_ip: vec!["0.0.0.0/0".parse().unwrap()],
                endpoint: "1.2.3.4:51820".parse().unwrap(),
            }],
        }
    }

    struct TempSettingsDir(PathBuf);

    impl TempSettingsDir {
        async fn new() -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "mullvad-custom-vpn-test-{}-{unique}",
                std::process::id()
            ));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempSettingsDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn test_load_missing_file() {
        let dir = TempSettingsDir::new().await;
        let path = dir.path().join(CUSTOM_VPN_SETTINGS_FILE);

        let persister = CustomVpnSettingsPersister::load(dir.path()).await;

        assert_eq!(persister.settings(), &CustomVpnSettings::default());
        assert!(!path.exists(), "load must not create the settings file");
    }

    #[tokio::test]
    async fn test_set_and_load_roundtrip() {
        let dir = TempSettingsDir::new().await;

        let mut persister = CustomVpnSettingsPersister::load(dir.path()).await;
        let settings = CustomVpnSettings {
            configs: vec![sample_config()],
        };
        persister.set(settings.clone()).await.unwrap();

        let reloaded = CustomVpnSettingsPersister::load(dir.path()).await;
        assert_eq!(reloaded.to_settings(), settings);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_written_file_has_restricted_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempSettingsDir::new().await;
        let path = dir.path().join(CUSTOM_VPN_SETTINGS_FILE);

        let mut persister = CustomVpnSettingsPersister::load(dir.path()).await;
        persister
            .set(CustomVpnSettings {
                configs: vec![sample_config()],
            })
            .await
            .unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[tokio::test]
    async fn test_invalid_file_falls_back_to_defaults() {
        let dir = TempSettingsDir::new().await;
        let path = dir.path().join(CUSTOM_VPN_SETTINGS_FILE);
        tokio::fs::write(&path, b"this is not json").await.unwrap();

        let persister = CustomVpnSettingsPersister::load(dir.path()).await;
        assert_eq!(persister.settings(), &CustomVpnSettings::default());
        assert_eq!(
            tokio::fs::read(&path).await.unwrap(),
            b"this is not json",
            "an unparsable file must be left untouched"
        );
    }

    #[tokio::test]
    async fn test_resolve_without_existing_key_fails() {
        let dir = TempSettingsDir::new().await;

        let persister = CustomVpnSettingsPersister::load(dir.path()).await;

        let config = sample_config();
        let update = CustomVpnConfigUpdate {
            enabled: config.enabled,
            name: config.name.clone(),
            tunnel: CustomVpnTunnelUpdate {
                private_key: None,
                ips: config.tunnel.ips.clone(),
            },
            peers: config.peers.clone(),
        };

        let resolved = update.resolve(persister.settings().configs.first());
        assert!(resolved.is_err());
        assert_eq!(
            persister.settings(),
            &CustomVpnSettings::default(),
            "the store must be unchanged after a failed resolve"
        );
    }
}
