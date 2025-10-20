//! This module implements fetching of information about app versions

use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::Context;
use tokio::fs;
#[cfg(test)]
use vec1::Vec1;

use crate::defaults;
use crate::format;
use crate::version::{VersionInfo, VersionParameters};

use super::version_provider::VersionInfoProvider;

use mullvad_api_constants::*;

/// Available platforms in the default metadata repository
#[derive(Debug, Clone, Copy)]
pub enum MetaRepositoryPlatform {
    Windows,
    Linux,
    Macos,
}

impl MetaRepositoryPlatform {
    /// Return the current platform
    pub fn current() -> Option<Self> {
        if cfg!(target_os = "windows") {
            Some(Self::Windows)
        } else if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else if cfg!(target_os = "macos") {
            Some(Self::Macos)
        } else {
            None
        }
    }

    /// Return complete URL used for the metadata
    pub fn url(&self) -> String {
        format!("{}/{}", defaults::RELEASES_URL, self.filename())
    }

    fn filename(&self) -> &str {
        match self {
            MetaRepositoryPlatform::Windows => "windows.json",
            MetaRepositoryPlatform::Linux => "linux.json",
            MetaRepositoryPlatform::Macos => "macos.json",
        }
    }
}

/// Obtain version data using a GET request
pub struct HttpVersionInfoProvider {
    /// Endpoint for GET request
    url: String,
    /// Optional host to resolve (to the IP) without DNS
    resolve: Option<(&'static str, IpAddr)>,
    /// Accepted root certificate. Defaults are used unless specified
    pinned_certificate: Option<reqwest::Certificate>,
    /// If set, the response metadata will be serialized and written to this path
    dump_to_path: Option<PathBuf>,
}

impl VersionInfoProvider for HttpVersionInfoProvider {
    async fn get_version_info(&self, params: &VersionParameters) -> anyhow::Result<VersionInfo> {
        let response = self.get_versions(params.lowest_metadata_version).await?;
        VersionInfo::try_from_response(params, response.signed)
    }

    fn set_metadata_dump_path(&mut self, path: PathBuf) {
        self.dump_to_path = Some(path);
    }
}

impl From<MetaRepositoryPlatform> for HttpVersionInfoProvider {
    /// Construct an [HttpVersionInfoProvider] for the given platform using reasonable defaults.
    ///
    /// By default, `pinned_certificate` will be set to the LE root certificate.
    fn from(platform: MetaRepositoryPlatform) -> Self {
        HttpVersionInfoProvider {
            url: platform.url(),
            resolve: Some((API_HOST_DEFAULT, API_IP_DEFAULT)),
            pinned_certificate: Some(defaults::PINNED_CERTIFICATE.clone()),
            dump_to_path: None,
        }
    }
}

impl HttpVersionInfoProvider {
    /// Maximum size of the GET response, in bytes
    const SIZE_LIMIT: usize = 1024 * 1024;

    /// Retrieve version metadata for the given platform using reasonable defaults.
    ///
    /// By default, `pinned_certificate` will be set to the LE root certificate, and
    /// `verifying_keys` will be set to the keys in `trusted-metadata-signing-keys`.
    pub async fn get_versions_for_platform(
        platform: MetaRepositoryPlatform,
        lowest_metadata_version: usize,
    ) -> anyhow::Result<format::SignedResponse> {
        HttpVersionInfoProvider::from(platform)
            .get_versions(lowest_metadata_version)
            .await
    }

    /// Download and verify signed data with sane defaults
    ///
    /// By default, `pinned_certificate` will be set to the LE root certificate, and
    /// and the keys in `trusted-metadata-signing-keys` will be used for verification.
    async fn get_versions(
        &self,
        lowest_metadata_version: usize,
    ) -> anyhow::Result<format::SignedResponse> {
        self.get_versions_inner(|raw_json| {
            format::SignedResponse::deserialize_and_verify(raw_json, lowest_metadata_version)
        })
        .await
    }

    /// Download and verify signed data with the given keys
    #[cfg(test)]
    async fn get_versions_with_keys(
        &self,
        lowest_metadata_version: usize,
        verifying_keys: &Vec1<format::key::VerifyingKey>,
    ) -> anyhow::Result<format::SignedResponse> {
        self.get_versions_inner(|raw_json| {
            format::SignedResponse::deserialize_and_verify_at_time(
                verifying_keys,
                raw_json,
                chrono::DateTime::UNIX_EPOCH,
                lowest_metadata_version,
            )
        })
        .await
    }

    async fn get_versions_inner(
        &self,
        deserialize_fn: impl FnOnce(&[u8]) -> anyhow::Result<format::SignedResponse>,
    ) -> anyhow::Result<format::SignedResponse> {
        let raw_json = Self::get(&self.url, self.pinned_certificate.clone(), self.resolve).await?;
        let signed_response = deserialize_fn(&raw_json)?;
        if let Some(path) = &self.dump_to_path {
            fs::write(path, raw_json)
                .await
                .context("Failed to save cache")?;
        }
        Ok(signed_response)
    }

    /// Retrieve the `latest.json` file.
    ///
    /// - `pinned_certificate` will be set to the LE root certificate.
    /// - DNS will be used to look up the URL.
    /// - The JSON response is not signed.
    pub async fn get_latest_versions_file() -> anyhow::Result<serde_json::Value> {
        Self::get(
            &format!("{}/latest.json", defaults::METADATA_URL),
            Some(defaults::PINNED_CERTIFICATE.clone()),
            None,
        )
        .await
        .and_then(|raw_json: Vec<u8>| Ok(String::from_utf8(raw_json)?))
        .and_then(|raw_json: String| Ok(serde_json::from_str(&raw_json)?))
        .context("Failed to get latest.json file")
    }

    /// Perform a simple GET request, with a size limit, and return it as bytes
    ///
    /// # Arguments
    /// `url` - URL to fetch
    /// `pinned_certificate` - Optional pinned certificate for TLS verification
    /// `resolve` - Optional host to resolve (to the IP) without DNS
    async fn get(
        url: &str,
        pinned_certificate: Option<reqwest::Certificate>,
        resolve: Option<(&'static str, IpAddr)>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut req_builder = reqwest::Client::builder();
        req_builder = req_builder.min_tls_version(reqwest::tls::Version::TLS_1_3);

        if let Some(pinned_certificate) = pinned_certificate {
            req_builder = req_builder
                .tls_built_in_root_certs(false)
                .add_root_certificate(pinned_certificate);
        }

        // Resolve name without DNS
        if let Some((host, addr)) = resolve {
            req_builder = req_builder.resolve(host, (addr, 0).into());
        }

        // Initiate GET request
        let mut req = req_builder
            .build()?
            .get(url)
            .send()
            .await
            .context("Failed to fetch version")?;

        // Fail if content length exceeds limit
        let content_len_limit = Self::SIZE_LIMIT.try_into().expect("Invalid size limit");
        if req.content_length() > Some(content_len_limit) {
            anyhow::bail!("Version info exceeded limit: {} bytes", Self::SIZE_LIMIT);
        }

        if let Err(err) = req.error_for_status_ref() {
            return Err(err).context("GET request failed");
        }

        let mut read_n = 0;
        let mut data = vec![];

        while let Some(chunk) = req.chunk().await.context("Failed to retrieve chunk")? {
            read_n += chunk.len();

            // Fail if content length exceeds limit
            if read_n > Self::SIZE_LIMIT {
                anyhow::bail!("Version info exceeded limit: {} bytes", Self::SIZE_LIMIT);
            }

            data.extend_from_slice(&chunk);
        }

        Ok(data)
    }
}

#[cfg(test)]
mod test {
    use async_tempfile::TempDir;
    use insta::assert_yaml_snapshot;
    use vec1::vec1;

    use super::*;
    use crate::format::SignedResponse;

    // These tests rely on `insta` for snapshot testing. If they fail due to snapshot assertions,
    // then most likely the snapshots need to be updated. The most convenient way to review
    // changes to, and update, snapshots is by running `cargo insta review`.

    /// Test HTTP version info provider
    ///
    /// We're not testing the correctness of [version] here, only the HTTP client
    #[tokio::test]
    async fn test_http_version_provider() -> anyhow::Result<()> {
        let valid_key =
            crate::format::key::VerifyingKey::from_hex(include_str!("../../test-pubkey"))
                .expect("valid key");
        let verifying_keys = vec1![valid_key];

        // Start HTTP server
        let mut server = mockito::Server::new_async().await;
        let _mock: mockito::Mock = server
            .mock("GET", "/version")
            // Respond with some version response payload
            .with_body(include_bytes!("../../test-version-response.json"))
            .create();

        // Resolve some host to our mockito server
        let host = "fakeurl.biz";
        let url = format!("http://{host}:{}/version", server.socket_address().port());
        let resolve = (host, server.socket_address().ip());

        let temp_dump_dir = TempDir::new().await.unwrap();
        let temp_dump = temp_dump_dir.join("metadata.json");

        // Construct query and provider
        let info_provider = HttpVersionInfoProvider {
            url: url.to_string(),
            pinned_certificate: None,
            resolve: Some(resolve),
            dump_to_path: Some(temp_dump.clone()),
        };

        let info = info_provider
            .get_versions_with_keys(0, &verifying_keys)
            .await
            .context("Expected valid version info")?;

        // Expect: Our query should yield some version response
        assert_yaml_snapshot!(info);

        // Expect: Dumped data should exist and look the same
        let cached_data = fs::read(temp_dump).await.expect("expected dumped info");
        let cached_info = SignedResponse::deserialize_and_verify_at_time(
            &verifying_keys,
            &cached_data,
            chrono::DateTime::UNIX_EPOCH,
            0,
        )
        .unwrap();
        assert_eq!(cached_info, info);

        Ok(())
    }
}
