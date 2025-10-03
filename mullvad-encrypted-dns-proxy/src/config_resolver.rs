//! Resolve valid proxy configurations via DoH.
//!
use core::fmt;
use std::net::IpAddr;
use std::time::Duration;

use hickory_resolver::config::*;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::{ResolveError, TokioResolver};
use tokio::time::error::Elapsed;

use crate::config;

/// The port to connect to the DoH resolvers over.
const RESOLVER_PORT: u16 = 443;
const DEFAULT_TIMEOUT: Duration = std::time::Duration::from_secs(10);

pub struct Nameserver {
    pub name: String,
    pub addr: Vec<IpAddr>,
}

#[derive(Debug)]
pub enum Error {
    ResolutionError(ResolveError),
    Timeout(Elapsed),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ResolutionError(err) => err.fmt(f),
            Error::Timeout(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ResolutionError(err) => err.source(),
            Self::Timeout(err) => err.source(),
        }
    }
}

/// Returns a set of well known public DoH resolvers. A sane default in many cases.
pub fn default_resolvers() -> Vec<Nameserver> {
    vec![
        Nameserver {
            name: "one.one.one.one".to_owned(),
            addr: vec!["1.1.1.1".parse().unwrap(), "1.0.0.1".parse().unwrap()],
        },
        Nameserver {
            name: "dns.quad9.net".to_owned(),
            addr: vec![
                "9.9.9.9".parse().unwrap(),
                "149.112.112.112".parse().unwrap(),
            ],
        },
    ]
}

/// Calls [resolve_configs] with a given `domain` using known DoH resolvers provided by [default_resolvers]
pub async fn resolve_default_config(domain: &str) -> Result<Vec<config::ProxyConfig>, Error> {
    resolve_configs(&default_resolvers(), domain).await
}

/// Looks up the `domain` towards the given `resolvers`, and try to deserialize all the returned
/// AAAA records into [`ProxyConfig`](config::ProxyConfig)s.
pub async fn resolve_configs(
    resolvers: &[Nameserver],
    domain: &str,
) -> Result<Vec<config::ProxyConfig>, Error> {
    let mut nameservers = ResolverConfig::new();
    for resolver in resolvers.iter() {
        let ns_config_group = NameServerConfigGroup::from_ips_https(
            &resolver.addr,
            RESOLVER_PORT,
            resolver.name.clone(),
            false,
        )
        .into_inner();
        for ns_config in ns_config_group {
            nameservers.add_name_server(ns_config);
        }
    }

    let resolver_config = {
        let mut config = ResolverOpts::default();
        config.tls_config = client_config_tls12();
        config.timeout = Duration::from_secs(5);
        config
    };
    resolve_config_with_resolverconfig(nameservers, resolver_config, domain, DEFAULT_TIMEOUT).await
}

pub async fn resolve_config_with_resolverconfig(
    resolver_config: ResolverConfig,
    options: ResolverOpts,
    domain: &str,
    timeout: Duration,
) -> Result<Vec<config::ProxyConfig>, Error> {
    let resolver =
        TokioResolver::builder_with_config(resolver_config, TokioConnectionProvider::default())
            .with_options(options)
            .build();
    let lookup = tokio::time::timeout(timeout, resolver.ipv6_lookup(domain))
        .await
        .map_err(Error::Timeout)?
        .map_err(Error::ResolutionError)?;

    let addrs = lookup.into_iter().map(|aaaa_record| aaaa_record.0);

    let mut proxy_configs = Vec::new();
    for addr in addrs {
        match config::ProxyConfig::try_from(addr) {
            Ok(proxy_config) => {
                log::trace!("IPv6 {addr} parsed into proxy config: {proxy_config:?}");
                proxy_configs.push(proxy_config);
            }
            Err(e) => log::error!("IPv6 {addr} fails to parse to a proxy config: {e}"),
        }
    }

    Ok(proxy_configs)
}

fn client_config_tls12() -> rustls::ClientConfig {
    let root_store = {
        let mut root_store = rustls::RootCertStore::empty();

        let trust_anchors =
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .map(|root_ca| rustls::pki_types::TrustAnchor {
                    subject: root_ca.subject.clone(),

                    subject_public_key_info: root_ca.subject_public_key_info.clone(),

                    name_constraints: root_ca.name_constraints.clone(),
                });

        root_store.extend(trust_anchors);

        root_store
    };

    // Ensure CryptoProvider is set for this process.

    let crypto_provider = rustls::crypto::ring::default_provider();

    if let Err(e) = crypto_provider.install_default() {
        log::error!("Crypto provider has already been installed: {e:?}");
    };

    rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

#[cfg(test)]
#[tokio::test]
async fn test_resolution() {
    let nameservers = vec![Nameserver {
        addr: vec!["1.1.1.1".parse().unwrap()],
        name: "one.one.one.one".to_owned(),
    }];

    let _ = resolve_configs(&nameservers, "frakta.eu").await.unwrap();
}

#[cfg(test)]
#[test]
fn default_resolvers_dont_panic() {
    let _ = default_resolvers();
}
