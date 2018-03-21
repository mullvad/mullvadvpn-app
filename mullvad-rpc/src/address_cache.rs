use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
    fallback_address_file: PathBuf,
}

impl AddressCache<SystemDnsResolver> {
    pub fn new(cache_dir: &Path, fallback_address_dir: &Path) -> Self {
        Self::with_dns_resolver(SystemDnsResolver, cache_dir, fallback_address_dir)
    }
}

impl<R: DnsResolver> AddressCache<R> {
    pub fn with_dns_resolver(
        dns_resolver: R,
        cache_dir: &Path,
        fallback_address_dir: &Path,
    ) -> Self {
        let cache = AddressCache {
            cache_file: cache_dir.join("api_address.txt"),
            fallback_address_file: fallback_address_dir.join("api_address.txt"),
            dns_resolver,
        };

        cache.create_initial_cache_if_needed();
        cache
    }

    pub fn api_address(&self) -> Option<String> {
        self.load_from_cache()
            .or_else(|_| self.resolve_into_cache())
            .map(|address| address.to_string())
            .ok()
    }

    fn create_initial_cache_if_needed(&self) {
        if self.load_from_cache().is_err() {
            if let Ok(address) = Self::load_from_file(&self.fallback_address_file) {
                let _ = self.store_in_cache(&address);
            }
        }
    }

    fn load_from_cache(&self) -> Result<IpAddr, io::Error> {
        lazy_static! {
            static ref MAX_CACHE_AGE: Duration = Duration::from_secs(3600);
        };

        let metadata = self.cache_file.metadata()?;
        let last_modified = metadata.modified()?;
        let cache_age = last_modified
            .elapsed()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to read cache age"))?;

        if cache_age > *MAX_CACHE_AGE {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Cache data is too old",
            ))
        } else {
            Self::load_from_file(&self.cache_file)
        }
    }

    fn load_from_file(file_path: &Path) -> Result<IpAddr, io::Error> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut address = String::new();

        reader.read_line(&mut address)?;

        address
            .trim()
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid address data"))
    }

    fn resolve_into_cache(&self) -> Result<IpAddr, io::Error> {
        let address = self.resolve_address()?;

        let _ = self.store_in_cache(&address);

        Ok(address)
    }

    fn resolve_address(&self) -> Result<IpAddr, io::Error> {
        self.dns_resolver
            .resolve(MASTER_API_HOST)
            .or_else(|_| Self::load_from_file(&self.fallback_address_file))
    }

    fn store_in_cache(&self, address: &IpAddr) -> Result<(), io::Error> {
        let mut cache_file = File::create(&self.cache_file)?;

        writeln!(cache_file, "{}", address)
    }
}

#[cfg(test)]
mod tests {
    extern crate filetime;
    extern crate tempdir;

    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Write};

    use self::filetime::FileTime;
    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_cached_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address: IpAddr = "127.0.0.1".parse().unwrap();

        {
            let cache_file_path = cache_dir.join("api_address.txt");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "{}", cached_address).unwrap();
        }

        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir, &resource_dir);
        let address = cache.api_address().unwrap();

        assert_eq!(address, cached_address.to_string());
    }

    #[test]
    fn ignores_old_cached_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let cache_file_path = cache_dir.join("api_address.txt");
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address: IpAddr = "127.0.0.1".parse().unwrap();

        {
            let mut cache_file = File::create(&cache_file_path).unwrap();
            writeln!(cache_file, "{}", cached_address).unwrap();
        }

        let cache_file_metadata = cache_file_path.metadata().unwrap();
        let last_access_time = FileTime::from_last_access_time(&cache_file_metadata);
        let fake_modification_time = FileTime::from_seconds_since_1970(100_000, 0);

        filetime::set_file_times(&cache_file_path, last_access_time, fake_modification_time)
            .unwrap();

        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir, &resource_dir);
        let address = cache.api_address().unwrap();

        assert_eq!(address, mock_resolver.address().to_string());
    }

    #[test]
    fn caches_resolved_ip() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir, &resource_dir);

        let _ = cache.api_address().unwrap();

        assert_eq!(
            get_cached_address(&cache_dir),
            mock_resolver.address().to_string()
        );
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let (temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir, &resource_dir);

        ::std::mem::drop(temp_dir);

        assert_eq!(
            cache.api_address().unwrap(),
            mock_resolver.address().to_string()
        );
    }

    #[test]
    fn uses_fallback_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let provided_address: IpAddr = "192.168.1.31".parse().unwrap();
        let cache = AddressCache::with_dns_resolver(FailingDnsResolver, &cache_dir, &resource_dir);

        {
            let fallback_file_path = resource_dir.join("api_address.txt");
            let mut fallback_file = File::create(fallback_file_path).unwrap();
            writeln!(fallback_file, "{}", provided_address).unwrap();
        }

        let address = cache.api_address().unwrap();

        assert_eq!(address, provided_address.to_string());
    }

    #[test]
    fn ignores_fallback_address_if_resolution_succeeds() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let provided_address: IpAddr = "192.168.1.31".parse().unwrap();
        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir, &resource_dir);

        {
            let fallback_file_path = resource_dir.join("api_address.txt");
            let mut fallback_file = File::create(fallback_file_path).unwrap();
            writeln!(fallback_file, "{}", provided_address).unwrap();
        }

        let address = cache.api_address().unwrap();

        assert_eq!(address, mock_resolver.address().to_string());
    }

    #[test]
    fn initially_populates_cache_with_fallback_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let provided_address: IpAddr = "192.168.1.31".parse().unwrap();

        {
            let fallback_file_path = resource_dir.join("api_address.txt");
            let mut fallback_file = File::create(fallback_file_path).unwrap();
            writeln!(fallback_file, "{}", provided_address).unwrap();
        }

        let _ = AddressCache::with_dns_resolver(FailingDnsResolver, &cache_dir, &resource_dir);

        assert_eq!(get_cached_address(&cache_dir), provided_address.to_string());
    }

    fn create_test_dirs() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let resource_dir = temp_dir.path().join("resource");

        fs::create_dir(&cache_dir).unwrap();
        fs::create_dir(&resource_dir).unwrap();

        (temp_dir, cache_dir, resource_dir)
    }

    fn get_cached_address(cache_dir: &Path) -> String {
        let cache_file_path = cache_dir.join("api_address.txt");

        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();

        cache_reader.read_line(&mut cached_address).unwrap();

        cached_address.trim().to_string()
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

    struct FailingDnsResolver;

    impl DnsResolver for FailingDnsResolver {
        fn resolve(&self, host: &str) -> Result<IpAddr, io::Error> {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to resolve address for {:?}", host),
            ))
        }
    }
}
