//! Built-in domain fronting access method configuration.

use std::{net::SocketAddr, str::FromStr, sync::LazyLock};

use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::{Level, instrument};

use crate::proxy::{ApiConnectionMode, ProxyConfig};

const FRONT_STR: &str = "https://www.phpmyadmin.net";
static FRONT: LazyLock<Uri> = LazyLock::new(|| FRONT_STR.parse().expect("Valid URI"));
const PROXY_HOST: &str = "1105015943.rsc.cdn77.org";
pub const SESSION_KEY: &str = "X-Mullvad-Session";

/// Whether we should use HTTP/2 (instead of HTTP/1.1) to talk to CDN77.
pub const USE_HTTP2: bool = true;

pub type DfConfig = domain_fronting::DomainFronting;

/// Resolve the built-in domain fronting configuration.
///
/// Performs DNS resolution of the front domain and returns the
/// corresponding [`ApiConnectionMode`].
pub async fn resolve() -> Option<ApiConnectionMode> {
    let default =
        DfConfig::new(FRONT.clone(), PROXY_HOST.to_string()).with_session_key(SESSION_KEY.into());
    resolve_with(&default).await
}

/// Resolve a custom domain fronting configuration.
///
/// Performs DNS resolution of the front domain and returns the
/// corresponding [`ApiConnectionMode`].
#[instrument(level = Level::TRACE, ret)]
pub async fn resolve_with(config: &DfConfig) -> Option<ApiConnectionMode> {
    let proxy_config = config
        .proxy_config()
        .await
        .inspect_err(|e| tracing::warn!("Failed to resolve domain fronting config: {e}"))
        .ok()?;

    let addr = DfConfigResolved {
        addr: proxy_config.addr,
        front: config.front().clone(),
        proxy_host: config.proxy_host().to_string(),
        session_key: config.session_key().to_string(),
    };

    Some(ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(
        addr,
    )))
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DfConfigResolved {
    /// Resolved address of the fronting domain.
    pub addr: SocketAddr,

    /// Domain that will be used to connect to a CDN.
    #[serde(serialize_with = "ser_display", deserialize_with = "de_from_str")]
    front: Uri,

    /// HTTP `HOST` header.
    proxy_host: String,

    session_key: String,
}

impl DfConfigResolved {
    pub fn config(&self) -> domain_fronting::DomainFronting {
        domain_fronting::DomainFronting::new(self.front.clone(), self.proxy_host.clone())
            .with_session_key(self.session_key.clone())
    }
}

fn ser_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::Display,
    S: Serializer,
{
    serializer.collect_str(value)
}

fn de_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
