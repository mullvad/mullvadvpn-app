//! This module implements fetching of information about app versions

use std::net::IpAddr;
use std::sync::Arc;

use super::defaults;
use anyhow::Context;
use rustls::{ClientConfig, RootCertStore, crypto::aws_lc_rs};

use mullvad_api_constants::*;

use mullvad_api::version::android::AndroidReleases;

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
    pub async fn get_releases() -> anyhow::Result<AndroidReleases> {
        let info_provider = HttpVersionInfoProvider {
            url: format!("{}{}", defaults::RELEASES_URL, "android.json"),
            resolve: Some((API_HOST_DEFAULT, API_IP_DEFAULT)),
        };
        info_provider.get_releases_inner().await
    }

    async fn get_releases_inner(&self) -> anyhow::Result<AndroidReleases> {
        let raw_json = Self::get(&self.url, self.resolve).await?;
        serde_json::from_slice(&raw_json).context("Failed to deserialize Android releases")
    }

    /// Retrieve the `latest.json` file for Android.
    pub async fn get_latest_versions_file() -> anyhow::Result<String> {
        Self::get(
            &format!("{}{}", defaults::METADATA_URL, "latest.json"),
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
    /// `resolve` - Optional host to resolve (to the IP) without DNS
    async fn get(url: &str, resolve: Option<(&'static str, IpAddr)>) -> anyhow::Result<Vec<u8>> {
        // reqwest is built without a bundled crypto provider, so feed it a
        // preconfigured aws-lc-rs rustls ClientConfig with TLS 1.3 enforced
        // and webpki roots as trust anchors.
        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls_config =
            ClientConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
                .with_protocol_versions(&[&rustls::version::TLS13])
                .expect("aws-lc-rs crypto provider should support TLS 1.3")
                .with_root_certificates(roots)
                .with_no_client_auth();
        let mut req_builder = reqwest::Client::builder().use_preconfigured_tls(tls_config);

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
