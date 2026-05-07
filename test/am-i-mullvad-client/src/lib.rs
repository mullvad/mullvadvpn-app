//! Minimal async client for the am.i.mullvad.net geoip endpoint.
//!
//! Pins the Let's Encrypt root certificate, enforces TLS 1.3 with X25519MLKEM768 only,
//! and disables SNI.

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Uri, header};
use hyper_util::client::legacy::Client;
use rustls::{
    ClientConfig,
    pki_types::{CertificateDer, pem::PemObject},
};
use serde::{Deserialize, Serialize};
use std::{
    net::IpAddr,
    sync::{Arc, LazyLock},
    time::Duration,
};

const LE_ROOT_CERT: &[u8] = include_bytes!("../../../mullvad-api/le_root_cert.pem");

const USER_AGENT: &str = "mullvad-app-testing";

/// Response body returned by the am.i.mullvad.net `/json` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmIMullvadResponse {
    /// Public IP the request was observed coming from.
    pub ip: IpAddr,
    /// `true` if `ip` is the exit IP of a Mullvad VPN relay.
    pub mullvad_exit_ip: bool,
    /// Hostname of the exit relay (e.g. `se-got-wg-001`) when `mullvad_exit_ip` is `true`,
    /// `None` otherwise.
    pub mullvad_exit_ip_hostname: Option<String>,
}

/// IP version selector for the am.i.mullvad.net endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    V4,
    V6,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The supplied `mullvad_host` produced a URL that does not parse.
    #[error("Invalid URL")]
    InvalidUrl(#[from] hyper::http::uri::InvalidUri),
    /// Connecting to the endpoint failed (TCP, TLS handshake, DNS, etc.).
    #[error("HTTP connection failed")]
    Connect(#[from] hyper_util::client::legacy::Error),
    /// The server replied with a non-2xx status code.
    #[error("Unexpected status code: {0}")]
    UnexpectedStatus(u16),
    /// Failed to read the response body off the wire after the headers were received.
    #[error("Failed to read response body")]
    ReadResponseBody(#[from] hyper::Error),
    /// The response body did not parse as the expected JSON shape.
    #[error("Failed to parse response body")]
    ParseResponseBody(#[from] serde_json::Error),
    /// The request did not complete within the supplied timeout.
    #[error("Request timed out")]
    Timeout,
}

/// Look up the current geoip status from `https://ipv4.am.i.{mullvad_host}/json`
/// (or the `ipv6.` variant when `ip_version` is `IpVersion::V6`).
///
/// # Errors
///
/// See [`Error`] variant documentation for different failure reasons
pub async fn geoip_lookup(
    mullvad_host: &str,
    ip_version: IpVersion,
    timeout: Duration,
) -> Result<AmIMullvadResponse, Error> {
    let prefix = match ip_version {
        IpVersion::V4 => "ipv4",
        IpVersion::V6 => "ipv6",
    };
    let uri = Uri::try_from(format!("https://{prefix}.am.i.{mullvad_host}/json"))?;
    tokio::time::timeout(timeout, http_get(uri))
        .await
        .map_err(|_| Error::Timeout)?
}

/// A lazily computed TLS client config with the settings we want
/// for TLS connections to mullvad https endpoints in general.
static CLIENT_CONFIG: LazyLock<ClientConfig> = LazyLock::new(|| {
    let provider = rustls::crypto::CryptoProvider {
        kx_groups: vec![rustls::crypto::aws_lc_rs::kx_group::X25519MLKEM768],
        ..rustls::crypto::aws_lc_rs::default_provider()
    };
    let mut config = ClientConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("aws-lc-rs crypto provider should support TLS 1.3")
        .with_root_certificates(create_pinned_cert_store())
        .with_no_client_auth();
    // The server certificate covers the relevant am.i.mullvad.net hostnames; SNI is omitted
    // so the destination subdomain is not visible in the ClientHello.
    config.enable_sni = false;
    config
});

async fn http_get(url: Uri) -> Result<AmIMullvadResponse, Error> {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(CLIENT_CONFIG.clone())
        .https_only()
        .enable_http2()
        .build();

    let client: Client<_, Full<Bytes>> =
        Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);
    let request = Request::get(url)
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, USER_AGENT)
        .body(Full::default())
        .expect("Static headers and a validated URI never fail to build");
    let response = client.request(request).await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatus(response.status().as_u16()));
    }
    let bytes = response.into_body().collect().await?.to_bytes();
    Ok(serde_json::from_slice(&bytes)?)
}

/// Creates and returns a certificate store with the single trusted bundled CA
fn create_pinned_cert_store() -> rustls::RootCertStore {
    let cert = CertificateDer::from_pem_slice(LE_ROOT_CERT)
        .expect("Bundled LE root cert PEM is malformed");
    let mut cert_store = rustls::RootCertStore::empty();
    cert_store
        .add(cert)
        .expect("Bundled LE root cert is invalid");
    cert_store
}
