use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

pub trait DnsResolver {
    fn resolve(&self, host: &str) -> io::Result<IpAddr>;
}

pub struct SystemDnsResolver;

impl DnsResolver for SystemDnsResolver {
    fn resolve(&self, host: &str) -> io::Result<IpAddr> {
        (host, 0)
            .to_socket_addrs()?
            .next()
            .map(|socket_address| socket_address.ip())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("Host not found: {}", host))
            })
    }
}

pub struct CachedDnsResolver<R: DnsResolver = SystemDnsResolver> {
    hostname: String,
    dns_resolver: R,
    cache_file: PathBuf,
    cached_address: Option<IpAddr>,
}

impl CachedDnsResolver<SystemDnsResolver> {
    pub fn new(hostname: String, cache_file: PathBuf) -> Self {
        Self::with_dns_resolver(SystemDnsResolver, hostname, cache_file)
    }
}

impl<R: DnsResolver> CachedDnsResolver<R> {
    pub fn with_dns_resolver(dns_resolver: R, hostname: String, cache_file: PathBuf) -> Self {
        let cached_address = Self::load_from_file(&cache_file).ok();

        CachedDnsResolver {
            hostname,
            dns_resolver,
            cache_file,
            cached_address,
        }
    }

    pub fn resolve(&self) -> Option<IpAddr> {
        self.cached_address
            .or_else(|| self.resolve_into_cache().ok())
    }

    fn load_from_file(file_path: &Path) -> io::Result<IpAddr> {
        let mut file = File::open(file_path)?;
        let mut address = String::new();

        file.read_to_string(&mut address)?;

        address
            .trim()
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid address data"))
    }

    fn resolve_into_cache(&self) -> io::Result<IpAddr> {
        let address = self.resolve_address()?;

        let _ = self.store_in_cache(address);

        Ok(address)
    }

    fn resolve_address(&self) -> io::Result<IpAddr> {
        self.dns_resolver.resolve(&self.hostname)
    }

    fn store_in_cache(&self, address: IpAddr) -> io::Result<()> {
        let mut cache_file = File::create(&self.cache_file)?;

        writeln!(cache_file, "{}", address)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_cached_address() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::with_address("192.168.1.206".parse().unwrap());
        let mock_resolver_was_called = mock_resolver.was_called_handle();
        let cached_address = "127.0.0.1".parse().unwrap();

        write_address(&cache_dir, cached_address);

        let cache = create_cached_dns_resolver(mock_resolver, &cache_dir);
        let address = cache.resolve().unwrap();

        assert!(!mock_resolver_was_called.load(Ordering::Acquire));
        assert_eq!(address, cached_address);
    }

    #[test]
    fn caches_resolved_ip() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let mock_address = "192.168.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);
        let cache = create_cached_dns_resolver(mock_resolver, &cache_dir);

        let address = cache.resolve().unwrap();

        assert_eq!(address, mock_address);
        assert_eq!(get_cached_address(&cache_dir), address.to_string());
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let (temp_dir, cache_dir) = create_test_dirs();
        let mock_address = "192.168.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);
        let cache = create_cached_dns_resolver(mock_resolver, &cache_dir);

        ::std::mem::drop(temp_dir);

        assert_eq!(cache.resolve().unwrap(), mock_address);
    }

    fn create_test_dirs() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let cache_dir = temp_dir.path().join("cache");

        fs::create_dir(&cache_dir).unwrap();

        (temp_dir, cache_dir)
    }

    fn write_address(dir: &Path, address: IpAddr) -> PathBuf {
        let file_path = dir.join("api_ip_address.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "{}", address).unwrap();

        file_path
    }

    fn get_cached_address(cache_dir: &Path) -> String {
        let cache_file_path = cache_dir.join("api_ip_address.txt");

        assert!(cache_file_path.exists());

        let mut cache_file = File::open(cache_file_path).unwrap();
        let mut cached_address = String::new();

        cache_file.read_to_string(&mut cached_address).unwrap();

        cached_address.trim().to_string()
    }

    fn create_cached_dns_resolver(
        mock_resolver: MockDnsResolver,
        cache_dir: &Path,
    ) -> CachedDnsResolver<MockDnsResolver> {
        let hostname = "dummy.host".to_owned();
        let filename = "api_ip_address.txt";
        let cache_file = cache_dir.join(filename);

        CachedDnsResolver::with_dns_resolver(mock_resolver, hostname, cache_file)
    }

    struct MockDnsResolver {
        address: Option<IpAddr>,
        called: Arc<AtomicBool>,
    }

    impl MockDnsResolver {
        pub fn with_address(address: IpAddr) -> Self {
            MockDnsResolver {
                address: Some(address),
                called: Arc::new(AtomicBool::new(false)),
            }
        }

        pub fn was_called_handle(&self) -> Arc<AtomicBool> {
            self.called.clone()
        }
    }

    impl DnsResolver for MockDnsResolver {
        fn resolve(&self, host: &str) -> io::Result<IpAddr> {
            self.called.store(true, Ordering::Release);

            self.address.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to resolve address for {:?}", host),
                )
            })
        }
    }
}
