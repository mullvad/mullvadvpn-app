use super::receive_confirmation;
use anyhow::Result;
use clap::{ValueEnum, builder::PossibleValue};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::settings::SettingsKeyList;

#[derive(Clone, Debug)]
pub struct SettingsKey(mullvad_types::settings::SettingsKey);

impl ValueEnum for SettingsKey {
    fn value_variants<'a>() -> &'a [Self] {
        use mullvad_types::settings::SettingsKey::*;
        &[
            Self(RelaySettings),
            Self(ObfuscationSettings),
            Self(CustomLists),
            Self(ApiAccessMethods),
            Self(UpdateDefaultLocation),
            Self(AllowLan),
            #[cfg(not(target_os = "android"))]
            Self(LockdownMode),
            Self(AutoConnect),
            Self(TunnelOptions),
            Self(RelayOverrides),
            Self(ShowBetaReleases),
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            Self(SplitTunnel),
            Self(Recents),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self.0 {
            mullvad_types::settings::SettingsKey::RelaySettings => {
                PossibleValue::new("relay-settings")
            }
            mullvad_types::settings::SettingsKey::ObfuscationSettings => {
                PossibleValue::new("anti-censorship")
            }
            mullvad_types::settings::SettingsKey::CustomLists => PossibleValue::new("custom-lists"),
            mullvad_types::settings::SettingsKey::ApiAccessMethods => {
                PossibleValue::new("api-access")
            }
            mullvad_types::settings::SettingsKey::UpdateDefaultLocation => {
                PossibleValue::new("update-default-location")
            }
            mullvad_types::settings::SettingsKey::AllowLan => PossibleValue::new("allow-lan"),
            #[cfg(not(target_os = "android"))]
            mullvad_types::settings::SettingsKey::LockdownMode => {
                PossibleValue::new("lockdown-mode")
            }
            mullvad_types::settings::SettingsKey::AutoConnect => PossibleValue::new("auto-connect"),
            mullvad_types::settings::SettingsKey::TunnelOptions => {
                PossibleValue::new("tunnel-options")
            }
            mullvad_types::settings::SettingsKey::RelayOverrides => {
                PossibleValue::new("relay-overrides")
            }
            mullvad_types::settings::SettingsKey::ShowBetaReleases => {
                PossibleValue::new("show-beta-releases")
            }
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            mullvad_types::settings::SettingsKey::SplitTunnel => PossibleValue::new("split-tunnel"),
            mullvad_types::settings::SettingsKey::Recents => PossibleValue::new("recents"),
        })
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
        keys: preserved.into_iter().map(|key| key.0).collect(),
    };
    rpc.reset_settings(preserved).await?;
    Ok(())
}
