//! Built-in domain fronting access method configuration.

use crate::proxy::{ApiConnectionMode, DomainFrontingConfig, ProxyConfig};

const FRONT: &str = "https://www.phpmyadmin.net";
const PROXY_HOST: &str = "1239602656.rsc.cdn77.org";
const AUTH: &str = "shared-secret";

/// Resolve the built-in domain fronting configuration.
///
/// Performs DNS resolution of the front domain and returns the
/// corresponding [`ApiConnectionMode`].
pub async fn resolve() -> Option<ApiConnectionMode> {
    // TODO: DNS lookup can be slow/flaky
    match DomainFrontingConfig::resolve(
        FRONT.parse().unwrap(),
        PROXY_HOST.to_string(),
        AUTH.to_string(),
    )
    .await
    {
        Ok(config) => Some(ApiConnectionMode::Proxied(ProxyConfig::DomainFronting(
            config,
        ))),
        Err(error) => {
            log::warn!("Failed to resolve domain fronting config: {error}");
            None
        }
    }
}
