use std::fs::File;
use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;

use super::MASTER_API_HOST;

pub trait DnsResolver {
    fn resolve(&self, host: &str) -> Result<IpAddr, io::Error>;
}

pub struct SystemDnsResolver;

impl DnsResolver for SystemDnsResolver {
    fn resolve(&self, host: &str) -> Result<IpAddr, io::Error> {
        (host, 0)
            .to_socket_addrs()?
            .next()
            .map(|socket_address| socket_address.ip())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("Host not found: {}", host))
            })
    }
}

pub struct AddressCache<R: DnsResolver = SystemDnsResolver> {
    cache_file: PathBuf,
    dns_resolver: R,
}

impl AddressCache<SystemDnsResolver> {
    pub fn new(cache_dir: &Path) -> Self {
        Self::with_dns_resolver(SystemDnsResolver, cache_dir)
    }
}

impl<R: DnsResolver> AddressCache<R> {
    pub fn with_dns_resolver(dns_resolver: R, cache_dir: &Path) -> Self {
        AddressCache {
            cache_file: cache_dir.join("api_address.json"),
            dns_resolver,
        }
    }

    pub fn api_address(&self) -> Option<String> {
        self.load_from_cache()
            .or_else(|_| self.resolve_into_cache())
            .map(|address| address.to_string())
            .ok()
    }

    fn load_from_cache(&self) -> Result<IpAddr, io::Error> {
        let cache_file = File::open(&self.cache_file)?;
        let address = serde_json::from_reader(cache_file)?;

        Ok(address)
    }

    fn resolve_into_cache(&self) -> Result<IpAddr, io::Error> {
        let address = self.resolve_address()?.into();

        let _ = self.store_in_cache(&address);

        Ok(address)
    }

    fn resolve_address(&self) -> Result<IpAddr, io::Error> {
        self.dns_resolver.resolve(MASTER_API_HOST)
    }

    fn store_in_cache(&self, address: &IpAddr) -> Result<(), io::Error> {
        let cache_file = File::create(&self.cache_file)?;

        serde_json::to_writer(&cache_file, &address)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};

    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_cached_address() {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address: IpAddr = "127.0.0.1".parse().unwrap();

        {
            let cache_file_path = temp_dir.path().join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let cache = AddressCache::with_dns_resolver(&mock_resolver, temp_dir.path());
        let address = cache.api_address().unwrap();

        assert_eq!(address, cached_address.to_string());
    }

    #[test]
    fn caches_resolved_ip() {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cache = AddressCache::with_dns_resolver(&mock_resolver, temp_dir.path());

        let address = cache.api_address().unwrap();

        let cache_file_path = temp_dir.path().join("api_address.json");
        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();
        cache_reader.read_line(&mut cached_address).unwrap();

        assert_eq!(address, mock_resolver.address().to_string());
        assert_eq!(cached_address, format!("\"{}\"", mock_resolver.address()));
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let temp_dir_path = TempDir::new("ip-cache-test").unwrap().path().to_path_buf();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cache = AddressCache::with_dns_resolver(&mock_resolver, &temp_dir_path);

        assert_eq!(
            cache.api_address().unwrap(),
            mock_resolver.address().to_string()
        );
    }

    struct MockDnsResolver {
        address: IpAddr,
    }

    impl MockDnsResolver {
        pub fn from_str(ip_address: &str) -> Self {
            MockDnsResolver {
                address: ip_address.parse().unwrap(),
            }
        }

        pub fn address(&self) -> &IpAddr {
            &self.address
        }
    }

    impl<'r> DnsResolver for &'r MockDnsResolver {
        fn resolve(&self, _host: &str) -> Result<IpAddr, io::Error> {
            Ok(self.address.clone())
        }
    }
}
