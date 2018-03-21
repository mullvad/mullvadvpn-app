use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};


static MAX_CACHE_AGE: Duration = Duration::from_secs(3600);
static EXPIRED_CACHE_TIMESTAMP: SystemTime = UNIX_EPOCH;


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
    cached_address: IpAddr,
    last_updated: SystemTime,
}

impl CachedDnsResolver<SystemDnsResolver> {
    pub fn new(hostname: String, cache_file: PathBuf, fallback_address: IpAddr) -> Self {
        Self::with_dns_resolver(SystemDnsResolver, hostname, cache_file, fallback_address)
    }
}

impl<R: DnsResolver> CachedDnsResolver<R> {
    pub fn with_dns_resolver(
        dns_resolver: R,
        hostname: String,
        cache_file: PathBuf,
        fallback_address: IpAddr,
    ) -> Self {
        let (cached_address, last_updated) =
            Self::load_initial_cached_address(&cache_file, fallback_address);

        CachedDnsResolver {
            hostname,
            dns_resolver,
            cache_file,
            cached_address,
            last_updated,
        }
    }

    pub fn resolve(&mut self) -> IpAddr {
        if let Ok(cache_age) = self.last_updated.elapsed() {
            if cache_age > MAX_CACHE_AGE {
                self.resolve_into_cache();
            }
        } else {
            self.resolve_into_cache();
        }

        self.cached_address
    }

    fn load_initial_cached_address(
        cache_file: &Path,
        fallback_address: IpAddr,
    ) -> (IpAddr, SystemTime) {
        match Self::load_from_file(cache_file) {
            Ok(previously_cached_address) => {
                let last_updated = Self::read_file_modification_time(cache_file)
                    .unwrap_or(EXPIRED_CACHE_TIMESTAMP);

                (previously_cached_address, last_updated)
            }
            Err(_) => (fallback_address, EXPIRED_CACHE_TIMESTAMP),
        }
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

    fn read_file_modification_time(cache_file: &Path) -> Option<SystemTime> {
        let metadata = cache_file.metadata().ok()?;

        metadata.modified().ok()
    }

    fn resolve_into_cache(&mut self) {
        if let Ok(address) = self.dns_resolver.resolve(&self.hostname) {
            self.cached_address = address;
            self.last_updated = SystemTime::now();
            self.update_cache_file();
        }
    }

    fn update_cache_file(&mut self) {
        if let Ok(mut cache_file) = File::create(&self.cache_file) {
            let _ = writeln!(cache_file, "{}", self.cached_address);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate filetime;
    extern crate tempdir;

    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use self::filetime::FileTime;
    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_previously_cached_address() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::with_address("192.168.1.206".parse().unwrap());
        let mock_resolver_was_called = mock_resolver.was_called_handle();
        let cached_address = "127.0.0.1".parse().unwrap();

        write_address(&cache_dir, cached_address);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);
        let address = cache.resolve();

        assert!(!mock_resolver_was_called.load(Ordering::Acquire));
        assert_eq!(address, cached_address);
    }

    #[test]
    fn old_cache_file_is_updated() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let cached_address = "127.0.0.1".parse().unwrap();
        let mock_address = "192.168.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let cache_file_path = write_address(&cache_dir, cached_address);

        make_file_old(&cache_file_path);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);
        let address = cache.resolve();

        assert_eq!(get_cached_address(&cache_dir), address.to_string());
        assert_eq!(address, mock_address);
    }

    #[test]
    fn old_cache_file_is_used_if_resolution_fails() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::that_fails();
        let cached_address = "127.0.0.1".parse().unwrap();

        let cache_file_path = write_address(&cache_dir, cached_address);

        make_file_old(&cache_file_path);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);
        let address = cache.resolve();

        assert_eq!(address, cached_address);
    }

    #[test]
    fn caches_resolved_ip() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let mock_address = "192.168.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);
        let address = cache.resolve();

        assert_eq!(address, mock_address);
        assert_eq!(get_cached_address(&cache_dir), address.to_string());
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let (temp_dir, cache_dir) = create_test_dirs();
        let mock_address = "192.168.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);

        ::std::mem::drop(temp_dir);

        assert_eq!(cache.resolve(), mock_address);
    }

    #[test]
    fn uses_fallback_address() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let fallback_address = "192.168.1.31".parse().unwrap();
        let mock_resolver = MockDnsResolver::that_fails();
        let mock_resolver_was_called = mock_resolver.was_called_handle();

        let mut cache =
            create_cached_dns_resolver(mock_resolver, &cache_dir, Some(fallback_address));
        let address = cache.resolve();

        assert!(mock_resolver_was_called.load(Ordering::Acquire));
        assert_eq!(address, fallback_address);
    }

    #[test]
    fn ignores_fallback_address_if_resolution_succeeds() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let fallback_address = "192.168.1.31".parse().unwrap();
        let mock_address = "192.168.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache =
            create_cached_dns_resolver(mock_resolver, &cache_dir, Some(fallback_address));
        let address = cache.resolve();

        assert_eq!(address, mock_address);
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

    fn make_file_old(file: &Path) {
        let file_metadata = file.metadata().unwrap();
        let last_access_time = FileTime::from_last_access_time(&file_metadata);
        let fake_modification_time = FileTime::from_seconds_since_1970(100_000, 0);

        filetime::set_file_times(&file, last_access_time, fake_modification_time).unwrap();
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
        fallback_address: Option<IpAddr>,
    ) -> CachedDnsResolver<MockDnsResolver> {
        let hostname = "dummy.host".to_owned();
        let filename = "api_ip_address.txt";
        let cache_file = cache_dir.join(filename);
        let fallback_address = fallback_address.unwrap_or(IpAddr::from([10, 0, 109, 91]));

        CachedDnsResolver::with_dns_resolver(mock_resolver, hostname, cache_file, fallback_address)
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

        pub fn that_fails() -> Self {
            MockDnsResolver {
                address: None,
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
