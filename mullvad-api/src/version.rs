use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use http::StatusCode;
use mullvad_update::version::{VersionInfo, VersionParameters, is_version_supported};

type AppVersion = String;

use super::APP_URL_PREFIX;
use super::rest;

#[derive(Clone)]
pub struct AppVersionProxy {
    handle: super::rest::MullvadRestHandle,
}

#[derive(serde::Deserialize, Debug)]
pub struct AppVersionResponse {
    pub supported: bool,
    pub latest: AppVersion,
    pub latest_stable: Option<AppVersion>,
    pub latest_beta: Option<AppVersion>,
}

/// Reply from `/app/releases/<platform>.json` endpoint
pub struct AppVersionResponse2 {
    /// Information about available versions for the current target
    pub version_info: VersionInfo,
    /// Index of the metadata version used to sign the response.
    /// Used to prevent replay/downgrade attacks.
    pub metadata_version: usize,
    /// Whether or not the current app version (mullvad_version::VERSION) is supported.
    pub current_version_supported: bool,
}

impl AppVersionProxy {
    /// Maximum size of `version_check_2` response
    const SIZE_LIMIT: usize = 1024 * 1024;

    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    pub fn version_check(
        &self,
        app_version: AppVersion,
        platform: &str,
        platform_version: String,
    ) -> impl Future<Output = Result<AppVersionResponse, rest::Error>> + use<> {
        let service = self.handle.service.clone();

        let path = format!("{APP_URL_PREFIX}/releases/{platform}/{app_version}");
        let request = self.handle.factory.get(&path);

        async move {
            let request = request?
                .expected_status(&[StatusCode::OK])
                .header("M-Platform-Version", &platform_version)?;
            let response = service.request(request).await?;
            response.deserialize().await
        }
    }

    /// Get versions from `/app/releases/<platform>.json`
    pub fn version_check_2(
        &self,
        platform: &str,
        architecture: mullvad_update::format::Architecture,
        rollout: f32,
        lowest_metadata_version: usize,
        platform_version: String,
    ) -> impl Future<Output = Result<AppVersionResponse2, rest::Error>> + use<> {
        let service = self.handle.service.clone();
        let path = format!("app/releases/{platform}.json");
        let request = self.handle.factory.get(&path);

        async move {
            let request = request?
                .expected_status(&[StatusCode::OK])
                .header("M-App-Version", mullvad_version::VERSION)?
                .header("M-Platform-Version", &platform_version)?;
            let response = service.request(request).await?;
            let bytes = response.body_with_max_size(Self::SIZE_LIMIT).await?;

            let response = mullvad_update::format::SignedResponse::deserialize_and_verify(
                &bytes,
                lowest_metadata_version,
            )
            .map_err(|err| rest::Error::FetchVersions(Arc::new(err)))?;

            let params = VersionParameters {
                architecture,
                rollout,
                // NOTE: On Linux, version metadata contains no installers
                allow_empty: cfg!(target_os = "linux"),
                lowest_metadata_version,
            };

            let current_version =
                mullvad_version::Version::from_str(mullvad_version::VERSION).unwrap();
            let current_version_supported = is_version_supported(current_version, &response.signed);

            let metadata_version = response.signed.metadata_version;
            Ok(AppVersionResponse2 {
                version_info: VersionInfo::try_from_response(&params, response.signed)
                    .map_err(Arc::new)
                    .map_err(rest::Error::FetchVersions)?,
                metadata_version,
                current_version_supported,
            })
        }
    }
}
