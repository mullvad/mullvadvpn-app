use std::fs::File;
use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
    fallback_address_file: Option<PathBuf>,
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
            fallback_address_file: None,
            dns_resolver,
        }
    }

    pub fn set_fallback_address_dir(&mut self, address_file_dir: &Path) {
        self.fallback_address_file = Some(address_file_dir.join("api_address.json"));
    }

    pub fn api_address(&self) -> Option<String> {
        self.load_from_cache()
            .or_else(|_| self.resolve_into_cache())
            .map(|address| address.to_string())
            .ok()
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
        let address = serde_json::from_reader(file)?;

        Ok(address)
    }

    fn resolve_into_cache(&self) -> Result<IpAddr, io::Error> {
        let address = self.resolve_address()?;

        let _ = self.store_in_cache(&address);

        Ok(address)
    }

    fn resolve_address(&self) -> Result<IpAddr, io::Error> {
        self.load_fallback_address()
            .or_else(|_| self.dns_resolver.resolve(MASTER_API_HOST))
    }

    fn load_fallback_address(&self) -> Result<IpAddr, io::Error> {
        if let Some(ref fallback_address_file) = self.fallback_address_file {
            Self::load_from_file(fallback_address_file)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no fallback address file specified",
            ))
        }
    }

    fn store_in_cache(&self, address: &IpAddr) -> Result<(), io::Error> {
        let cache_file = File::create(&self.cache_file)?;

        serde_json::to_writer(&cache_file, &address)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
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
        let (_temp_dir, cache_dir, _) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address: IpAddr = "127.0.0.1".parse().unwrap();

        {
            let cache_file_path = cache_dir.join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir);
        let address = cache.api_address().unwrap();

        assert_eq!(address, cached_address.to_string());
    }

    #[test]
    fn ignores_old_cached_address() {
        let (_temp_dir, cache_dir, _) = create_test_dirs();
        let cache_file_path = cache_dir.join("api_address.json");
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address: IpAddr = "127.0.0.1".parse().unwrap();

        {
            let mut cache_file = File::create(&cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let cache_file_metadata = cache_file_path.metadata().unwrap();
        let last_access_time = FileTime::from_last_access_time(&cache_file_metadata);
        let fake_modification_time = FileTime::from_seconds_since_1970(100_000, 0);

        filetime::set_file_times(&cache_file_path, last_access_time, fake_modification_time)
            .unwrap();

        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir);
        let address = cache.api_address().unwrap();

        assert_eq!(address, mock_resolver.address().to_string());
    }

    #[test]
    fn caches_resolved_ip() {
        let (_temp_dir, cache_dir, _) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir);

        let address = cache.api_address().unwrap();

        let cache_file_path = cache_dir.join("api_address.json");
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
        let (temp_dir, cache_dir, _) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir);

        ::std::mem::drop(temp_dir);

        assert_eq!(
            cache.api_address().unwrap(),
            mock_resolver.address().to_string()
        );
    }

    #[test]
    fn uses_fallback_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let provided_address: IpAddr = "192.168.1.31".parse().unwrap();

        {
            let fallback_file_path = cache_dir.join("api_address.json");
            let mut fallback_file = File::create(fallback_file_path).unwrap();
            writeln!(fallback_file, "\"{}\"", provided_address).unwrap();
        }

        let mut cache = AddressCache::with_dns_resolver(&mock_resolver, &cache_dir);
        cache.set_fallback_address_dir(&resource_dir);

        let address = cache.api_address().unwrap();

        assert_eq!(address, provided_address.to_string());
    }

    fn create_test_dirs() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let resource_dir = temp_dir.path().join("resource");

        fs::create_dir(&cache_dir).unwrap();
        fs::create_dir(&resource_dir).unwrap();

        (temp_dir, cache_dir, resource_dir)
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
