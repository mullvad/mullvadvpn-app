//! Built-in domain fronting access method configuration.

use std::{cell::LazyCell, net::SocketAddr};

use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use tracing::{Level, instrument};

use crate::proxy::{ApiConnectionMode, ProxyConfig};

const FRONT_STR: &str = "https://www.phpmyadmin.net";
static FRONT: LazyLock<Uri> = LazyLock::new(|| FRONT_STR.parse().expect("Valid URI"));
const PROXY_HOST: &str = "1239602656.rsc.cdn77.org";
pub const USE_HTTP2: bool = true;

/// Resolve the built-in domain fronting configuration.
///
/// Performs DNS resolution of the front domain and returns the
/// corresponding [`ApiConnectionMode`].
#[instrument(level = Level::TRACE, ret)]
pub async fn resolve() -> Option<ApiConnectionMode> {
    // TODO: DNS lookup can be slow/flaky
    let df = domain_fronting::DomainFronting::new(FRONT.clone(), PROXY_HOST.to_string());
    let proxy_config = df
        .proxy_config()
        .await
        .inspect_err(|e| tracing::warn!("Failed to resolve domain fronting config: {e}"))
        .ok()?;

    let addr = DomainFrontingAddr {
        addr: proxy_config.addr,
        _front: FRONT.to_string(),
    };

    Some(ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(
        addr,
    )))
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomainFrontingAddr {
    /// Resolved address of the fronting domain.
    pub addr: SocketAddr,

    /// Domain that will be used to connect to a CDN.
    ///
    /// This is used as a cache-key, to guard against [`FRONT`] changing on crate updates.
    #[serde(rename = "front", deserialize_with = "deserialize_front_uri")]
    _front: String,
}

impl DomainFrontingAddr {
    pub fn config(&self) -> domain_fronting::DomainFronting {
        domain_fronting::DomainFronting::new(FRONT.clone(), PROXY_HOST.to_string())
    }
}

/// Deserialize a fronting URI and assert that it equals [`FRONT`].
pub fn deserialize_front_uri<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let front: &str = Deserialize::deserialize(deserializer)?;
    if front != FRONT_STR {
        return Err(D::Error::custom("Invalid fronting domain (Stale cache)"));
    }
    Ok(FRONT_STR.to_string())
}
