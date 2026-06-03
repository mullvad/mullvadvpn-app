use crate::types::proto;

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
            mullvad_types::settings::SettingsKey::LockdownMode => LockdownModeKey,
            mullvad_types::settings::SettingsKey::AutoConnect => AutoConnect,
            mullvad_types::settings::SettingsKey::TunnelOptions => TunnelOptions,
            mullvad_types::settings::SettingsKey::RelayOverrides => RelayOverrides,
            mullvad_types::settings::SettingsKey::ShowBetaReleases => ShowBetaReleases,
            mullvad_types::settings::SettingsKey::SplitTunnel => SplitTunnel,
            mullvad_types::settings::SettingsKey::Recents => Recents,
        }
    }
}

impl From<proto::SettingsKey> for mullvad_types::settings::SettingsKey {
    fn from(key: proto::SettingsKey) -> Self {
        match key {
            proto::SettingsKey::RelaySettings => Self::RelaySettings,
            proto::SettingsKey::AllowLan => Self::AllowLan,
            proto::SettingsKey::LockdownModeKey => Self::LockdownMode,
            proto::SettingsKey::AutoConnect => Self::AutoConnect,
            proto::SettingsKey::TunnelOptions => Self::TunnelOptions,
            proto::SettingsKey::ShowBetaReleases => Self::ShowBetaReleases,
            proto::SettingsKey::SplitTunnel => Self::SplitTunnel,
            proto::SettingsKey::ObfuscationSettings => Self::ObfuscationSettings,
            proto::SettingsKey::CustomLists => Self::CustomLists,
            proto::SettingsKey::ApiAccessMethods => Self::ApiAccessMethods,
            proto::SettingsKey::RelayOverrides => Self::RelayOverrides,
            proto::SettingsKey::Recents => Self::Recents,
            proto::SettingsKey::UpdateDefaultLocation => Self::UpdateDefaultLocation,
        }
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

impl From<proto::SettingsKeyList> for mullvad_types::settings::SettingsKeyList {
    fn from(keys: proto::SettingsKeyList) -> Self {
        mullvad_types::settings::SettingsKeyList {
            keys: keys.keys().map(From::from).collect(),
        }
    }
}
