use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use http::StatusCode;
use http::header;
use mullvad_update::version::rollout::Rollout;
use mullvad_update::version::{VersionInfo, VersionParameters, is_version_supported};

type AppVersion = String;

use super::APP_URL_PREFIX;
use super::rest;

#[derive(Clone)]
pub struct AppVersionProxy {
    handle: super::rest::MullvadRestHandle,
}

#[derive(Debug)]
pub struct AppVersionResponse {
    response: AppVersionResponseRaw,
    pub etag: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct AppVersionResponseRaw {
    supported: bool,
    latest: AppVersion,
    latest_stable: Option<AppVersion>,
    latest_beta: Option<AppVersion>,
}

impl AppVersionResponse {
    pub const fn supported(&self) -> bool {
        self.response.supported
    }

    pub const fn latest_stable(&self) -> Option<&AppVersion> {
        self.response.latest_stable.as_ref()
    }

    pub const fn latest_beta(&self) -> Option<&AppVersion> {
        self.response.latest_beta.as_ref()
    }
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
    /// ETag for the response
    pub etag: Option<String>,
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
        platform_version: Option<String>,
        etag: Option<String>,
    ) -> impl Future<Output = Result<Option<AppVersionResponse>, rest::Error>> + use<> {
        let service = self.handle.service.clone();

        let path = format!("{APP_URL_PREFIX}/releases/{platform}/{app_version}");
        let request = self.handle.factory.get(&path);

        async move {
            let mut request = request?.expected_status(&[StatusCode::NOT_MODIFIED, StatusCode::OK]);
            if let Some(platform_version) = platform_version {
                request = request.header("M-Platform-Version", &platform_version)?;
            }
            if let Some(ref tag) = etag {
                request = request.header(header::IF_NONE_MATCH, tag)?;
            }
            let response = service.request(request).await?;
            if etag.is_some() && response.status() == StatusCode::NOT_MODIFIED {
                return Ok(None);
            }
            let etag = Self::extract_etag(&response);
            let deserialized: AppVersionResponseRaw = response.deserialize().await?;
            let _ = deserialized.latest; // we do not use this

            Ok(Some(AppVersionResponse {
                response: deserialized,
                etag,
            }))
        }
    }

    /// Get versions from `/app/releases/<platform>.json`
    ///
    /// This returns `None` if the server responds with 304 (version is same as etag).
    pub fn version_check_2(
        &self,
        platform: &str,
        architecture: mullvad_update::format::Architecture,
        lowest_metadata_version: usize,
        platform_version: Option<String>,
        rollout: Rollout,
        etag: Option<String>,
    ) -> impl Future<Output = Result<Option<AppVersionResponse2>, rest::Error>> + use<> {
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
            let current_version_supported =
                is_version_supported(current_version, &response.signed.releases);

            let metadata_version = response.signed.metadata_version;
            Ok(Some(AppVersionResponse2 {
                version_info: VersionInfo::try_from_response(&params, response.signed)
                    .map_err(Arc::new)
                    .map_err(rest::Error::FetchVersions)?,
                metadata_version,
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
