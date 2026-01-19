use super::rest;
#[cfg(target_os = "android")]
use anyhow::Context;
use http::StatusCode;
use http::header;
#[cfg(not(target_os = "android"))]
use mullvad_update::{
    format::response::SignedResponse,
    version::{Rollout, VersionInfo, VersionParameters, is_version_supported},
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppVersionProxy {
    handle: super::rest::MullvadRestHandle,
}

/// Reply from `/app/releases/<platform>.json` endpoint
pub struct AppVersionResponse {
    /// Information about available versions for the current target
    #[cfg(not(target_os = "android"))]
    pub version_info: VersionInfo,
    /// Index of the metadata version used to sign the response.
    /// Used to prevent replay/downgrade attacks.
    #[cfg(not(target_os = "android"))]
    pub metadata_version: usize,
    /// Whether or not the current app version (mullvad_version::VERSION) is supported.
    pub current_version_supported: bool,
    /// ETag for the response
    pub etag: Option<String>,
}

/// Android releases
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[cfg(target_os = "android")]
pub struct AndroidReleases {
    /// Available app releases
    pub releases: Vec<Release>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[cfg(target_os = "android")]
pub struct Release {
    /// Mullvad app version
    pub version: mullvad_version::Version,
}

impl AppVersionProxy {
    /// Maximum size of `version_check` response
    const SIZE_LIMIT: usize = 1024 * 1024;

    pub fn new(handle: rest::MullvadRestHandle) -> Self {
        Self { handle }
    }

    /// Get versions from `/app/releases/<platform>.json`
    ///
    /// This returns `None` if the server responds with 304 (version is same as etag).
    #[cfg(not(target_os = "android"))]
    pub fn version_check(
        &self,
        platform: &str,
        architecture: mullvad_update::format::Architecture,
        lowest_metadata_version: usize,
        platform_version: Option<String>,
        rollout: Rollout,
        etag: Option<String>,
    ) -> impl Future<Output = Result<Option<AppVersionResponse>, rest::Error>> + use<> {
        let service = self.handle.service.clone();
        let path = format!("app/releases/{platform}.json");
        let request = self.handle.factory.get(&path);

        async move {
            let mut request = request?.expected_status(&[StatusCode::NOT_MODIFIED, StatusCode::OK]);
            if let Some(platform_version) = platform_version {
                request = request
                    .header(
                        "M-App-Version",
                        &sanitize_header_value(mullvad_version::VERSION),
                    )?
                    .header(
                        "M-Platform-Version",
                        &sanitize_header_value(&platform_version),
                    )?;
            }
            if let Some(ref tag) = etag {
                request = request.header(header::IF_NONE_MATCH, tag)?;
            }
            let response = service.request(request).await?;
            if etag.is_some() && response.status() == StatusCode::NOT_MODIFIED {
                return Ok(None);
            }
            let etag = Self::extract_etag(&response);

            let bytes = response.body_with_max_size(Self::SIZE_LIMIT).await?;

            let response = SignedResponse::deserialize_and_verify(&bytes, lowest_metadata_version)
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
            Ok(Some(AppVersionResponse {
                version_info: VersionInfo::try_from_response(&params, response.signed)
                    .map_err(Arc::new)
                    .map_err(rest::Error::FetchVersions)?,
                metadata_version,
                current_version_supported,
                etag,
            }))
        }
    }

    #[cfg(target_os = "android")]
    pub fn version_check_android(
        &self,
        platform_version: Option<String>,
        etag: Option<String>,
    ) -> impl Future<Output = Result<Option<AppVersionResponse>, rest::Error>> + use<> {
        let service = self.handle.service.clone();
        let path = "app/releases/android.json".to_string();
        let request = self.handle.factory.get(&path);

        async move {
            let mut request = request?.expected_status(&[StatusCode::NOT_MODIFIED, StatusCode::OK]);
            if let Some(platform_version) = platform_version {
                request = request
                    .header(
                        "M-App-Version",
                        &sanitize_header_value(mullvad_version::VERSION),
                    )?
                    .header(
                        "M-Platform-Version",
                        &sanitize_header_value(&platform_version),
                    )?;
            }
            if let Some(ref tag) = etag {
                request = request.header(header::IF_NONE_MATCH, tag)?;
            }
            let response = service.request(request).await?;
            if etag.is_some() && response.status() == StatusCode::NOT_MODIFIED {
                return Ok(None);
            }
            let etag = Self::extract_etag(&response);

            let bytes = response.body_with_max_size(Self::SIZE_LIMIT).await?;

            let response: AndroidReleases = serde_json::from_slice(&bytes)
                .context("Invalid version JSON")
                .map_err(|err| rest::Error::FetchVersions(Arc::new(err)))?;

            let current_version =
                mullvad_version::Version::from_str(mullvad_version::VERSION).unwrap();
            let current_version_supported =
                is_version_supported_android(&current_version, &response);

            Ok(Some(AppVersionResponse {
                current_version_supported,
                etag,
            }))
        }
    }

    pub fn extract_etag(response: &rest::Response<hyper::body::Incoming>) -> Option<String> {
        response
            .headers()
            .get(header::ETAG)
            .and_then(|tag| match tag.to_str() {
                Ok(tag) => Some(tag.to_string()),
                Err(_) => {
                    log::error!("Ignoring invalid tag from server: {:?}", tag.as_bytes());
                    None
                }
            })
    }
}

pub fn is_version_supported_android(
    current_version: &mullvad_version::Version,
    response: &AndroidReleases,
) -> bool {
    response
        .releases
        .iter()
        .any(|release| release.version == *current_version)
}

// This function makes a string conform to the allowed characters and length of header values.
// Here's the rule it needs to implement: [A-Za-z0-9_.-]{1,64}
fn sanitize_header_value(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_whitespace() { '_' } else { c })
        .filter(|&c| c.is_ascii_alphanumeric() || "_.-".contains(c))
        .take(64)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sanitize_header_value() {
        assert_eq!(sanitize_header_value("2025.5"), "2025.5");
        assert_eq!(sanitize_header_value("Fedora Linux"), "Fedora_Linux");
        assert_eq!(sanitize_header_value("macOS 26.1"), "macOS_26.1");
        assert_eq!(sanitize_header_value("Déjà vu OS"), "Dj_vu_OS");

        let long_value =
            "abcdefghijklmnopqrstuvxyzabcdefghijklmnopqrstuvxyzabcdefghijklmnopqrstuvxyz";
        let mut truncated_long_value = long_value.to_owned();
        truncated_long_value.truncate(64);
        assert_eq!(sanitize_header_value(long_value), truncated_long_value);
    }
}
