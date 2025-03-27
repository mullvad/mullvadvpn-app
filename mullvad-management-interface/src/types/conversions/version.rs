use std::path::PathBuf;

use crate::types::proto;
use mullvad_types::version::*;

use super::FromProtobufTypeError;

impl From<AppVersionInfo> for proto::AppVersionInfo {
    fn from(version_info: AppVersionInfo) -> Self {
        Self {
            supported: version_info.current_version_supported,
            suggested_upgrade: version_info
                .suggested_upgrade
                .map(proto::SuggestedUpgrade::from),
        }
    }
}

impl TryFrom<proto::AppVersionInfo> for AppVersionInfo {
    type Error = FromProtobufTypeError;

    fn try_from(version_info: proto::AppVersionInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            current_version_supported: version_info.supported,
            suggested_upgrade: version_info
                .suggested_upgrade
                .map(SuggestedUpgrade::try_from)
                .transpose()?,
        })
    }
}

impl From<SuggestedUpgrade> for proto::SuggestedUpgrade {
    fn from(suggested_upgrade: SuggestedUpgrade) -> Self {
        Self {
            version: suggested_upgrade.version.to_string(),
            changelog: suggested_upgrade.changelog,
            verified_installer_path: suggested_upgrade
                .verified_installer_path
                .and_then(|path| path.to_str().map(str::to_owned)),
        }
    }
}

impl TryFrom<proto::SuggestedUpgrade> for SuggestedUpgrade {
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

impl From<AppUpgradeEvent> for proto::AppUpgradeEvent {
    fn from(upgrade_event: AppUpgradeEvent) -> Self {
        type ProtoEvent = proto::app_upgrade_event::Event;

        let event = match upgrade_event {
            AppUpgradeEvent::DownloadStarting => {
                ProtoEvent::DownloadStarting(proto::AppUpgradeDownloadStarting {})
            }
            AppUpgradeEvent::DownloadProgress(progress) => {
                ProtoEvent::DownloadProgress(progress.into())
            }
            AppUpgradeEvent::VerifyingInstaller => {
                ProtoEvent::VerifyingInstaller(proto::AppUpgradeVerifyingInstaller {})
            }
            AppUpgradeEvent::VerifiedInstaller => {
                ProtoEvent::VerifiedInstaller(proto::AppUpgradeVerifiedInstaller {})
            }
            AppUpgradeEvent::Aborted => ProtoEvent::UpgradeAborted(proto::AppUpgradeAborted {}),
            AppUpgradeEvent::Error(app_upgrade_error) => {
                ProtoEvent::Error(app_upgrade_error.into())
            }
        };
        Self { event: Some(event) }
    }
}

impl TryFrom<proto::AppUpgradeEvent> for AppUpgradeEvent {
    type Error = FromProtobufTypeError;

    fn try_from(upgrade_event: proto::AppUpgradeEvent) -> Result<Self, FromProtobufTypeError> {
        type ProtoEvent = proto::app_upgrade_event::Event;

        let event = upgrade_event
            .event
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "Non-existent AppUpgradeEvent",
            ))?;

        let event = match event {
            ProtoEvent::DownloadStarting(_starting) => AppUpgradeEvent::DownloadStarting,
            ProtoEvent::DownloadProgress(progress) => {
                let progress = AppUpgradeDownloadProgress::try_from(progress)?;
                AppUpgradeEvent::DownloadProgress(progress)
            }
            ProtoEvent::VerifyingInstaller(_verifying) => AppUpgradeEvent::VerifyingInstaller,
            ProtoEvent::VerifiedInstaller(_verified) => AppUpgradeEvent::VerifiedInstaller,
            ProtoEvent::UpgradeAborted(_aborted) => AppUpgradeEvent::Aborted,
            ProtoEvent::Error(error) => {
                let error = AppUpgradeError::try_from(error)?;
                AppUpgradeEvent::Error(error)
            }
        };
        Ok(event)
    }
}

impl From<AppUpgradeDownloadProgress> for proto::AppUpgradeDownloadProgress {
    fn from(value: AppUpgradeDownloadProgress) -> Self {
        // TODO: Is it acceptable to unwrap in this case?
        // From the docs: Converts a std::time::Duration to a Duration, failing if the duration is too large.
        let time_left = prost_types::Duration::try_from(value.time_left).unwrap();
        proto::AppUpgradeDownloadProgress {
            server: value.server,
            progress: value.progress,
            time_left: Some(time_left),
        }
    }
}

impl TryFrom<proto::AppUpgradeDownloadProgress> for AppUpgradeDownloadProgress {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::AppUpgradeDownloadProgress) -> Result<Self, Self::Error> {
        let Some(time_left) = value.time_left else {
            return Err(FromProtobufTypeError::InvalidArgument(
                "Non-existent AppUpgradeDownloadProgress::time_left",
            ));
        };
        // TODO: Is it acceptable to unwrap in this case?
        // From the docs: Converts a Duration to a std::time::Duration, failing if the duration is negative.
        let time_left = std::time::Duration::try_from(time_left).unwrap();
        let progress = AppUpgradeDownloadProgress {
            server: value.server,
            progress: value.progress,
            time_left,
        };
        Ok(progress)
    }
}

impl From<AppUpgradeError> for proto::AppUpgradeError {
    fn from(value: AppUpgradeError) -> Self {
        type ProtoError = proto::app_upgrade_error::Error;
        match value {
            AppUpgradeError::GeneralError => proto::AppUpgradeError {
                error: ProtoError::GeneralError as i32,
            },
            AppUpgradeError::DownloadFailed => proto::AppUpgradeError {
                error: ProtoError::DownloadFailed as i32,
            },
            AppUpgradeError::VerificationFailed => proto::AppUpgradeError {
                // TODO: Spelling mistake! Should be VerificationFailed xd
                error: ProtoError::VerficationFailed as i32,
            },
        }
    }
}

impl TryFrom<proto::AppUpgradeError> for AppUpgradeError {
    type Error = FromProtobufTypeError;

    fn try_from(value: proto::AppUpgradeError) -> Result<Self, Self::Error> {
        type ProtoError = proto::app_upgrade_error::Error;
        let Ok(error) = ProtoError::try_from(value.error) else {
            return Err(FromProtobufTypeError::InvalidArgument(
                "invalid AppUpgradeError",
            ));
        };
        match error {
            ProtoError::GeneralError => Ok(AppUpgradeError::GeneralError),
            ProtoError::DownloadFailed => Ok(AppUpgradeError::DownloadFailed),
            ProtoError::VerficationFailed => Ok(AppUpgradeError::VerificationFailed),
        }
    }
}
