use crate::types::{FromProtobufTypeError, proto};

impl From<mullvad_types::settings::SettingsKey> for proto::SettingsKey {
    fn from(key: mullvad_types::settings::SettingsKey) -> Self {
        use proto::SettingsKey::*;
        match key {
            mullvad_types::settings::SettingsKey::RelaySettings => RelaySettings,
            mullvad_types::settings::SettingsKey::ObfuscationSettings => ObfuscationSettings,
            mullvad_types::settings::SettingsKey::CustomLists => CustomLists,
            mullvad_types::settings::SettingsKey::ApiAccessMethods => ApiAccessMethods,
            mullvad_types::settings::SettingsKey::UpdateDefaultLocation => UpdateDefaultLocation,
            mullvad_types::settings::SettingsKey::AllowLan => AllowLan,
            #[cfg(not(target_os = "android"))]
            mullvad_types::settings::SettingsKey::LockdownMode => LockdownModeKey,
            mullvad_types::settings::SettingsKey::AutoConnect => AutoConnect,
            mullvad_types::settings::SettingsKey::TunnelOptions => TunnelOptions,
            mullvad_types::settings::SettingsKey::RelayOverrides => RelayOverrides,
            mullvad_types::settings::SettingsKey::ShowBetaReleases => ShowBetaReleases,
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            mullvad_types::settings::SettingsKey::SplitTunnel => SplitTunnel,
            mullvad_types::settings::SettingsKey::Recents => Recents,
        }
    }
}

impl TryFrom<proto::SettingsKey> for mullvad_types::settings::SettingsKey {
    type Error = FromProtobufTypeError;

    fn try_from(key: proto::SettingsKey) -> Result<Self, Self::Error> {
        Ok(match key {
            proto::SettingsKey::RelaySettings => Self::RelaySettings,
            proto::SettingsKey::AllowLan => Self::AllowLan,
            #[cfg(not(target_os = "android"))]
            proto::SettingsKey::LockdownModeKey => Self::LockdownMode,
            #[cfg(target_os = "android")]
            proto::SettingsKey::LockdownModeKey => {
                return Err(FromProtobufTypeError::invalid_argument(
                    "lockdown mode not supported on this platform",
                ));
            }
            proto::SettingsKey::AutoConnect => Self::AutoConnect,
            proto::SettingsKey::TunnelOptions => Self::TunnelOptions,
            proto::SettingsKey::ShowBetaReleases => Self::ShowBetaReleases,
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            proto::SettingsKey::SplitTunnel => Self::SplitTunnel,
            #[cfg(not(any(windows, target_os = "android", target_os = "macos")))]
            proto::SettingsKey::SplitTunnel => {
                return Err(FromProtobufTypeError::invalid_argument(
                    "split tunnel not supported on this platform",
                ));
            }
            proto::SettingsKey::ObfuscationSettings => Self::ObfuscationSettings,
            proto::SettingsKey::CustomLists => Self::CustomLists,
            proto::SettingsKey::ApiAccessMethods => Self::ApiAccessMethods,
            proto::SettingsKey::RelayOverrides => Self::RelayOverrides,
            proto::SettingsKey::Recents => Self::Recents,
            proto::SettingsKey::UpdateDefaultLocation => Self::UpdateDefaultLocation,
        })
    }
}

impl From<mullvad_types::settings::SettingsKeyList> for proto::SettingsKeyList {
    fn from(keys: mullvad_types::settings::SettingsKeyList) -> Self {
        proto::SettingsKeyList {
            keys: keys
                .keys
                .into_iter()
                .map(|key| proto::SettingsKey::from(key) as i32)
                .collect(),
        }
    }
}

impl TryFrom<proto::SettingsKeyList> for mullvad_types::settings::SettingsKeyList {
    type Error = FromProtobufTypeError;

    fn try_from(keys: proto::SettingsKeyList) -> Result<Self, Self::Error> {
        let keys = keys
            .keys()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(mullvad_types::settings::SettingsKeyList { keys })
    }
}
