#![cfg(target_os = "android")]
//! See [AndroidDnsResolver].

use async_trait::async_trait;
use hickory_resolver::{TokioResolver, config::*, name_server::TokioConnectionProvider};
use mullvad_api::DnsResolver;
use std::{io, net::SocketAddr};
use talpid_core::connectivity_listener::ConnectivityListener;

/// A non-blocking DNS resolver. The default resolver uses `getaddrinfo`, which often prevents the
/// tokio runtime from being dropped, since it waits indefinitely on blocking threads. This is
/// particularly bad on Android, so we use a non-blocking resolver instead.
pub struct AndroidDnsResolver {
    connectivity_listener: ConnectivityListener,
}

impl AndroidDnsResolver {
    pub fn new(connectivity_listener: ConnectivityListener) -> Self {
        Self {
            connectivity_listener,
        }
    }
}

#[async_trait]
impl DnsResolver for AndroidDnsResolver {
    async fn resolve(&self, host: String) -> io::Result<Vec<SocketAddr>> {
        let ips = self
            .connectivity_listener
            .current_dns_servers()
            .map_err(|err| {
                io::Error::other(format!("Failed to retrieve current servers: {err}"))
            })?;
        let group = NameServerConfigGroup::from_ips_clear(&ips, 53, false);

        let config = ResolverConfig::from_parts(None, vec![], group);
        let mut opts = ResolverOpts::default();
        opts.attempts = 0;
        opts.use_hosts_file = ResolveHosts::Never;
        let provider = TokioConnectionProvider::default();
        let resolver = TokioResolver::builder_with_config(config, provider)
            .with_options(opts)
            .build();

        let lookup = resolver
            .lookup_ip(host)
            .await
            .map_err(|err| io::Error::other(format!("lookup_ip failed: {err}")))?;

        Ok(lookup.into_iter().map(|ip| (ip, 0).into()).collect())
    }
}
