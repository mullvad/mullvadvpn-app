use super::receive_confirmation;
use anyhow::Result;
use clap::ValueEnum;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::settings::SettingsKeyList;

#[derive(Clone, Debug, ValueEnum)]
pub enum SettingsKey {
    RelaySettings,
    ObfuscationSettings,
    CustomLists,
    ApiAccessMethods,
    UpdateDefaultLocation,
    AllowLan,
    LockdownMode,
    AutoConnect,
    TunnelOptions,
    RelayOverrides,
    ShowBetaReleases,
    SplitTunnel,
    Recents,
}

impl From<SettingsKey> for mullvad_types::settings::SettingsKey {
    fn from(key: SettingsKey) -> Self {
        match key {
            SettingsKey::RelaySettings => mullvad_types::settings::SettingsKey::RelaySettings,
            SettingsKey::ObfuscationSettings => {
                mullvad_types::settings::SettingsKey::ObfuscationSettings
            }
            SettingsKey::CustomLists => mullvad_types::settings::SettingsKey::CustomLists,
            SettingsKey::ApiAccessMethods => mullvad_types::settings::SettingsKey::ApiAccessMethods,
            SettingsKey::UpdateDefaultLocation => {
                mullvad_types::settings::SettingsKey::UpdateDefaultLocation
            }
            SettingsKey::AllowLan => mullvad_types::settings::SettingsKey::AllowLan,
            SettingsKey::LockdownMode => mullvad_types::settings::SettingsKey::LockdownMode,
            SettingsKey::AutoConnect => mullvad_types::settings::SettingsKey::AutoConnect,
            SettingsKey::TunnelOptions => mullvad_types::settings::SettingsKey::TunnelOptions,
            SettingsKey::RelayOverrides => mullvad_types::settings::SettingsKey::RelayOverrides,
            SettingsKey::ShowBetaReleases => mullvad_types::settings::SettingsKey::ShowBetaReleases,
            SettingsKey::SplitTunnel => mullvad_types::settings::SettingsKey::SplitTunnel,
            SettingsKey::Recents => mullvad_types::settings::SettingsKey::Recents,
        }
    }
}

pub async fn handle_factory_reset(assume_yes: bool) -> Result<()> {
    if !assume_yes && !receive_confirmation("Are you sure you want to disconnect, log out, delete all settings, logs and cache files for the Mullvad VPN system service?", false).await {
        return Ok(());
    }
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.factory_reset().await?;
    #[cfg(target_os = "linux")]
    println!("If you're running systemd, to remove all logs, you must use journalctl");
    Ok(())
}

pub async fn handle_settings_reset(assume_yes: bool, preserved: Vec<SettingsKey>) -> Result<()> {
    if !assume_yes
        && !receive_confirmation(
            "Are you sure you want to reset all settings to the default?",
            false,
        )
        .await
    {
        return Ok(());
    }
    let mut rpc = MullvadProxyClient::new().await?;
    let preserved = SettingsKeyList {
        keys: preserved.into_iter().map(From::from).collect(),
    };
    rpc.reset_settings(preserved).await?;
    Ok(())
}
