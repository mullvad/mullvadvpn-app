//! This module implements fetching of information about app versions

use std::net::IpAddr;

use super::defaults;
use anyhow::Context;

use mullvad_api_constants::*;

/// Obtain version data using a GET request
pub struct HttpVersionInfoProvider {
    /// Endpoint for GET request
    url: String,
    /// Optional host to resolve (to the IP) without DNS
    resolve: Option<(&'static str, IpAddr)>,
}

impl HttpVersionInfoProvider {
    /// Maximum size of the GET response, in bytes
    const SIZE_LIMIT: usize = 1024 * 1024;

    /// Retrieve released versions for Android.
    pub async fn get_releases() -> anyhow::Result<mullvad_api::version::AndroidReleases> {
        let info_provider = HttpVersionInfoProvider {
            url: format!("{}{}", defaults::RELEASES_URL, "android.json"),
            resolve: Some((API_HOST_DEFAULT, API_IP_DEFAULT)),
        };
        info_provider.get_releases_inner().await
    }

    async fn get_releases_inner(&self) -> anyhow::Result<mullvad_api::version::AndroidReleases> {
        let raw_json = Self::get(&self.url, None, self.resolve).await?;
        serde_json::from_slice(&raw_json).context("Failed to deserialize Android releases")
    }

    /// Retrieve the `latest.json` file for Android.
    pub async fn get_latest_versions_file() -> anyhow::Result<String> {
        Self::get(
            &format!("{}{}", defaults::METADATA_URL, "latest.json"),
            None,
            None,
        )
        .await
        .and_then(|raw_json: Vec<u8>| Ok(String::from_utf8(raw_json)?))
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
