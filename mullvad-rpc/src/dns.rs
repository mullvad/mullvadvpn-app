use std::fs::File;
use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::ops::DerefMut;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde_json;

pub trait DnsResolver {
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr>;

    fn fallback_with<R>(self, resolver: R) -> DnsResolverWithFallback<Self, R>
    where
        R: DnsResolver,
        Self: Sized,
    {
        DnsResolverWithFallback::new(self, resolver)
    }
}

impl DnsResolver for Box<DnsResolver> {
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr> {
        self.deref_mut().resolve(host)
    }
}

pub struct DirectDnsResolver;

impl DnsResolver for DirectDnsResolver {
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr> {
        (host, 0)
            .to_socket_addrs()?
            .next()
            .map(|socket_address| socket_address.ip())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("could not resolve hostname {}", host),
                )
            })
    }
}

pub struct StaticDnsResolver {
    address_file: PathBuf,
}

impl StaticDnsResolver {
    pub fn new(address_file: PathBuf) -> Self {
        StaticDnsResolver { address_file }
    }
}

impl DnsResolver for StaticDnsResolver {
    fn resolve(&mut self, _host: &str) -> io::Result<IpAddr> {
        let file = File::open(&self.address_file)?;
        let address = serde_json::from_reader(file)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

        Ok(address)
    }
}

pub struct CachedDnsResolver<R: DnsResolver = DirectDnsResolver> {
    cache_file: PathBuf,
    resolver: R,
    max_time_in_cache: Duration,
}

impl CachedDnsResolver<DirectDnsResolver> {
    pub fn new(cache_file: PathBuf) -> Self {
        Self::with_resolver(cache_file, DirectDnsResolver)
    }
}

impl<R: DnsResolver> CachedDnsResolver<R> {
    pub fn with_resolver(cache_file: PathBuf, resolver: R) -> Self {
        CachedDnsResolver {
            cache_file,
            resolver,
            max_time_in_cache: Duration::from_secs(24 * 60 * 60),
        }
    }

    pub fn set_max_time_in_cache(&mut self, time: Duration) {
        self.max_time_in_cache = time;
    }

    fn cache_is_old(&self) -> bool {
        if let Ok(last_modified) = self.cache_file_last_modified() {
            if let Ok(time) = last_modified.elapsed() {
                return time > self.max_time_in_cache;
            }
        }

        // Assume it needs to be reloaded
        true
    }

    fn cache_file_last_modified(&self) -> io::Result<SystemTime> {
        self.cache_file.metadata()?.modified()
    }

    fn resolve_into_cache(&mut self, host: &str) -> io::Result<IpAddr> {
        let address = self.resolver.resolve(host)?;

        // Return resolved address even if storing in cache fails
        let _ = self.store_in_cache(address);

        Ok(address)
    }

    fn read_from_cache(&mut self) -> io::Result<IpAddr> {
        let file = File::open(&self.cache_file)?;
        let address = serde_json::from_reader(file)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

        Ok(address)
    }

    fn store_in_cache(&mut self, address: IpAddr) -> io::Result<()> {
        let file = File::create(&self.cache_file)?;

        serde_json::to_writer(&file, &address)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }
}

impl<R: DnsResolver> DnsResolver for CachedDnsResolver<R> {
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr> {
        if self.cache_is_old() {
            self.resolve_into_cache(host)
        } else {
            self.read_from_cache()
                .or_else(|_| self.resolve_into_cache(host))
        }
    }
}

pub struct DnsResolverWithFallback<A, B>
where
    A: DnsResolver,
    B: DnsResolver,
{
    primary: A,
    fallback: B,
}

impl<A, B> DnsResolverWithFallback<A, B>
where
    A: DnsResolver,
    B: DnsResolver,
{
    pub fn new(primary: A, fallback: B) -> Self {
        DnsResolverWithFallback { primary, fallback }
    }
}

impl<A, B> DnsResolver for DnsResolverWithFallback<A, B>
where
    A: DnsResolver,
    B: DnsResolver,
{
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr> {
        self.primary
            .resolve(host)
            .or_else(|_| self.fallback.resolve(host))
    }
}
