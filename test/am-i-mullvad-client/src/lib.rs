//! Minimal async client for the am.i.mullvad.net geoip endpoint.
//!
//! The connection is configured by [`mullvad_tls_client::api`], the same way
//! the daemon configures its own connection check to these hosts.

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Uri, header};
use hyper_util::client::legacy::Client;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};

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

async fn http_get(url: Uri) -> Result<AmIMullvadResponse, Error> {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(mullvad_tls_client::api().clone())
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
