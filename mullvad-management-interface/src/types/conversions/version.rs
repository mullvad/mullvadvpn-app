use crate::types::proto;

impl From<mullvad_types::version::AppVersionInfo> for proto::AppVersionInfo {
    fn from(version_info: mullvad_types::version::AppVersionInfo) -> Self {
        Self {
            supported: version_info.supported,
            latest_stable: version_info.latest_stable,
            latest_beta: version_info.latest_beta,
            suggested_upgrade: version_info.suggested_upgrade.unwrap_or_default(),
        }
    }
}

impl From<proto::AppVersionInfo> for mullvad_types::version::AppVersionInfo {
    fn from(version_info: proto::AppVersionInfo) -> Self {
        Self {
            supported: version_info.supported,
            latest_stable: version_info.latest_stable,
            latest_beta: version_info.latest_beta,
            suggested_upgrade: if version_info.suggested_upgrade.is_empty() {
                None
            } else {
                Some(version_info.suggested_upgrade)
            },
            // NOTE: This field is meaningless when derived from the gRPC type
            wg_migration_threshold: f32::NAN,
        }
    }
}
