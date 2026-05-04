//! Resolve valid proxy configurations via DoH.
//!
use crate::config;
use core::fmt;
use hickory_resolver::{
    TokioResolver,
    config::*,
    net::{NetError, runtime::TokioRuntimeProvider},
};
use rustls::ClientConfig;
use std::{net::IpAddr, time::Duration};
use tokio::time::error::Elapsed;

const DEFAULT_TIMEOUT: Duration = std::time::Duration::from_secs(10);

pub struct Nameserver {
    pub name: String,
    pub addr: Vec<IpAddr>,
}

#[derive(Debug)]
pub enum Error {
    ProtocolError(NetError),
    Timeout(Elapsed),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ProtocolError(err) => err.fmt(f),
            Error::Timeout(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProtocolError(err) => err.source(),
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
    let mut config = ResolverConfig::default();
    for resolver in resolvers.iter() {
        let servers = ServerGroup {
            ips: resolver.addr.as_slice(),
            server_name: &resolver.name,
            // De facto DoH URL: https://www.rfc-editor.org/rfc/rfc8484
            path: "/dns-query",
        };
        for server in servers.https() {
            config.add_name_server(server);
        }
    }

    resolve_config_with_resolverconfig(config, ResolverOpts::default(), domain, DEFAULT_TIMEOUT)
        .await
}

pub async fn resolve_config_with_resolverconfig(
    resolver_config: ResolverConfig,
    options: ResolverOpts,
    domain: &str,
    timeout: Duration,
) -> Result<Vec<config::ProxyConfig>, Error> {
    let provider = TokioRuntimeProvider::default();
    let resolver = TokioResolver::builder_with_config(resolver_config, provider)
        .with_options(options)
        .with_tls_config(client_config_tls12())
        .build()
        .map_err(Error::ProtocolError)?;

    let lookup = tokio::time::timeout(timeout, resolver.lookup_ip(domain))
        .await
        .map_err(Error::Timeout)?
        .map_err(Error::ProtocolError)?;

    let addrs = lookup.iter().filter_map(|addr| match addr {
        IpAddr::V4(_) => None,
        IpAddr::V6(addr) => Some(addr),
    });

    let mut proxy_configs = Vec::new();
    for addr in addrs {
        match config::ProxyConfig::try_from(addr) {
            Ok(proxy_config) => {
                log::trace!("IPv6 {addr} parsed into proxy config: {proxy_config:?}");
                proxy_configs.push(proxy_config);
            }
            Err(config::Error::XorV1Unsupported) => continue, // ignore deprecated configs
            Err(e) => log::error!("IPv6 {addr} fails to parse to a proxy config: {e}"),
        }
    }

    Ok(proxy_configs)
}

fn client_config_tls12() -> ClientConfig {
    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    ClientConfig::builder()
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
