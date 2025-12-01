//! Built-in domain fronting access method configuration.

use crate::proxy::{ApiConnectionMode, DomainFrontingConfig, ProxyConfig};

const FRONT: &str = "www.phpmyadmin.net";
const PROXY_HOST: &str = "1105015943.rsc.cdn77.org";
const SESSION_HEADER: &str = "X-Mullvad-Session";

/// Resolve the built-in domain fronting configuration.
///
/// Performs DNS resolution of the front domain and returns the
/// corresponding [`ApiConnectionMode`].
pub async fn resolve() -> Option<ApiConnectionMode> {
    match DomainFrontingConfig::resolve(
        FRONT.to_string(),
        PROXY_HOST.to_string(),
        SESSION_HEADER.to_string(),
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
