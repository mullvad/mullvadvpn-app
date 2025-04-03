use std::future::Future;
use std::sync::Arc;

use http::StatusCode;
use mullvad_types::version::AppVersion;
use mullvad_update::version::{VersionInfo, VersionParameters};

use super::rest;
use super::APP_URL_PREFIX;

#[derive(Clone)]
pub struct AppVersionProxy {
    handle: super::rest::MullvadRestHandle,
}

#[derive(serde::Deserialize, Debug)]
pub struct AppVersionResponse {
    pub supported: bool,
    pub latest: AppVersion,
    pub latest_stable: Option<AppVersion>,
    pub latest_beta: AppVersion,
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
    ) -> impl Future<Output = Result<VersionInfo, rest::Error>> + use<> {
        let service = self.handle.service.clone();
        let path = format!("app/releases/{platform}.json");
        let request = self.handle.factory.get(&path);

        async move {
            let request = request?.expected_status(&[StatusCode::OK]);
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
                lowest_metadata_version,
            };

            VersionInfo::try_from_response(&params, response.signed)
                .map_err(Arc::new)
                .map_err(rest::Error::FetchVersions)
        }
    }
}
