#![cfg(target_os = "android")]
//! A non-blocking DNS resolver. `getaddrinfo` tends to prevent the tokio runtime from being
//! dropped, since it waits indefinitely on blocking threads. This is particularly bad on Android,
//! so we use a non-blocking resolver instead.

use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use mullvad_api::DnsResolver;
use std::{future::Future, io, net::IpAddr, pin::Pin};

pub struct AndroidDnsResolver {
    connectivity_listener: talpid_core::connectivity_listener::ConnectivityListener,
}

impl AndroidDnsResolver {
    pub fn new(
        connectivity_listener: talpid_core::connectivity_listener::ConnectivityListener,
    ) -> Self {
        Self {
            connectivity_listener,
        }
    }
}

impl DnsResolver for AndroidDnsResolver {
    fn resolve(
        &self,
        host: String,
    ) -> Pin<Box<dyn Future<Output = io::Result<Vec<IpAddr>>> + Send>> {
        let ips = self.connectivity_listener.current_dns_servers();

        Box::pin(async move {
            let ips = ips.map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to retrieve current servers: {err}"),
                )
            })?;
            let group = NameServerConfigGroup::from_ips_clear(&ips, 53, false);

            let config = ResolverConfig::from_parts(None, vec![], group);
            let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());

            let lookup = resolver.lookup_ip(host).await.map_err(|err| {
                io::Error::new(io::ErrorKind::Other, format!("lookup_ip failed: {err}"))
            })?;

            Ok(lookup.into_iter().collect())
        })
    }
}
