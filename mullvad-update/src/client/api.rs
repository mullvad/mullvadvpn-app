//! This module implements fetching of information about app versions

use anyhow::Context;
use vec1::Vec1;

use crate::format;
use crate::version::{VersionInfo, VersionParameters};

/// See [module-level](self) docs.
#[async_trait::async_trait]
pub trait VersionInfoProvider {
    /// Return info about the stable version
    async fn get_version_info(&self, params: VersionParameters) -> anyhow::Result<VersionInfo>;
}

/// Obtain version data using a GET request
pub struct HttpVersionInfoProvider {
    /// Endpoint for GET request
    pub url: String,
    /// Accepted root certificate. Defaults are used unless specified
    pub pinned_certificate: Option<reqwest::Certificate>,
    /// Key to use for verifying the response
    pub verifying_keys: Vec1<format::key::VerifyingKey>,
}

#[async_trait::async_trait]
impl VersionInfoProvider for HttpVersionInfoProvider {
    async fn get_version_info(&self, params: VersionParameters) -> anyhow::Result<VersionInfo> {
        let response = self.get_versions(params.lowest_metadata_version).await?;
        VersionInfo::try_from_response(&params, response.signed)
    }
}

impl HttpVersionInfoProvider {
    /// Maximum size of the GET response, in bytes
    const SIZE_LIMIT: usize = 1024 * 1024;

    /// Construct a provider for the Mullvad API using [crate::keys::TRUSTED_METADATA_SIGNING_PUBKEYS]
    pub fn trusted_provider(platform: &str) -> Self {
        /// Pinned root certificate used when fetching version metadata
        const PINNED_CERTIFICATE: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

        /// Base URL for pulling metadata. Actual JSON files should be stored at `<base
        /// url>/<platform>.json`
        const META_REPOSITORY_URL: &str = "https://api.mullvad.net/app/releases/";

        let url = format!("{META_REPOSITORY_URL}/{platform}.json");

        let cert = reqwest::Certificate::from_pem(PINNED_CERTIFICATE).expect("invalid cert");

        HttpVersionInfoProvider {
            url,
            pinned_certificate: Some(cert),
            verifying_keys: crate::keys::TRUSTED_METADATA_SIGNING_PUBKEYS.clone(),
        }
    }

    /// Download and verify signed data
    pub async fn get_versions(
        &self,
        lowest_metadata_version: usize,
    ) -> anyhow::Result<format::SignedResponse> {
        let raw_json = Self::get(&self.url, self.pinned_certificate.clone()).await?;
        let response = format::SignedResponse::deserialize_and_verify(
            &self.verifying_keys,
            &raw_json,
            lowest_metadata_version,
        )?;
        Ok(response)
    }

    /// Perform a simple GET request, with a size limit, and return it as bytes
    async fn get(
        url: &str,
        pinned_certificate: Option<reqwest::Certificate>,
    ) -> anyhow::Result<Vec<u8>> {
        let mut req_builder = reqwest::Client::builder();
        req_builder = req_builder.min_tls_version(reqwest::tls::Version::TLS_1_3);

        if let Some(pinned_certificate) = pinned_certificate {
            req_builder = req_builder
                .tls_built_in_root_certs(false)
                .add_root_certificate(pinned_certificate);
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
    use insta::assert_yaml_snapshot;
    use vec1::vec1;

    use crate::version::VersionArchitecture;

    use super::*;

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
        let _mock = server
            .mock("GET", "/version")
            // Respond with some version response payload
            .with_body(include_bytes!("../../test-version-response.json"))
            .create();

        let url = format!("{}/version", server.url());

        // Construct query and provider
        let params = VersionParameters {
            architecture: VersionArchitecture::X86,
            rollout: 1.,
            lowest_metadata_version: 0,
        };
        let info_provider = HttpVersionInfoProvider {
            url,
            pinned_certificate: None,
            verifying_keys,
        };

        let info = info_provider
            .get_version_info(params)
            .await
            .context("Expected valid version info")?;

        // Expect: Our query should yield some version response
        assert_yaml_snapshot!(info);

        Ok(())
    }
}
