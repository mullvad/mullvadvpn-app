use std::path::PathBuf;

use crate::types::proto;

use super::FromProtobufTypeError;

impl From<mullvad_types::version::AppVersionInfo> for proto::AppVersionInfo {
    fn from(version_info: mullvad_types::version::AppVersionInfo) -> Self {
        Self {
            supported: version_info.supported,
            suggested_upgrade: version_info
                .suggested_upgrade
                .map(proto::SuggestedUpgrade::from),
        }
    }
}

impl TryFrom<proto::AppVersionInfo> for mullvad_types::version::AppVersionInfo {
    type Error = FromProtobufTypeError;

    fn try_from(version_info: proto::AppVersionInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            supported: version_info.supported,
            suggested_upgrade: version_info
                .suggested_upgrade
                .map(mullvad_types::version::SuggestedUpgrade::try_from)
                .transpose()?,
        })
    }
}

impl From<mullvad_types::version::SuggestedUpgrade> for proto::SuggestedUpgrade {
    fn from(suggested_upgrade: mullvad_types::version::SuggestedUpgrade) -> Self {
        Self {
            version: suggested_upgrade.version.to_string(),
            changelog: suggested_upgrade.changelog,
            verified_installer_path: suggested_upgrade
                .verified_installer_path
                .and_then(|path| path.to_str().map(str::to_owned)),
        }
    }
}

impl TryFrom<proto::SuggestedUpgrade> for mullvad_types::version::SuggestedUpgrade {
    type Error = FromProtobufTypeError;

    fn try_from(suggested_upgrade: proto::SuggestedUpgrade) -> Result<Self, Self::Error> {
        // TODO: we probably don't need to convert in this direction
        let version = suggested_upgrade.version.parse().map_err(|_err| {
            FromProtobufTypeError::InvalidArgument("invalid Mullvad app version")
        })?;
        let verified_installer_path = suggested_upgrade
            .verified_installer_path
            .map(|path| PathBuf::from(&path));

        Ok(Self {
            version,
            changelog: suggested_upgrade.changelog,
            verified_installer_path,
        })
    }
}
