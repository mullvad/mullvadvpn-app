use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};


lazy_static! {
    static ref MAX_CACHE_AGE: Duration = Duration::from_secs(3600);
}


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
    last_updated: Instant,
}

impl CachedDnsResolver<SystemDnsResolver> {
    pub fn new(
        hostname: String,
        cache_dir: &Path,
        fallback_dir: &Path,
        filename: &str,
    ) -> io::Result<Self> {
        Self::with_dns_resolver(
            SystemDnsResolver,
            hostname,
            cache_dir,
            fallback_dir,
            filename,
        )
    }
}

impl<R: DnsResolver> CachedDnsResolver<R> {
    pub fn with_dns_resolver(
        dns_resolver: R,
        hostname: String,
        cache_dir: &Path,
        fallback_dir: &Path,
        filename: &str,
    ) -> io::Result<Self> {
        let cache_file = cache_dir.join(filename);
        let fallback_file = fallback_dir.join(filename);

        let (cached_address, last_updated) =
            Self::load_initial_cached_address(&cache_file, &fallback_file)?;

        Ok(CachedDnsResolver {
            hostname,
            dns_resolver,
            cache_file,
            cached_address,
            last_updated,
        })
    }

    pub fn resolve(&mut self) -> IpAddr {
        if self.last_updated.elapsed() > *MAX_CACHE_AGE {
            self.resolve_into_cache();
        }

        self.cached_address
    }

    fn load_initial_cached_address(
        cache_file: &Path,
        fallback_file: &Path,
    ) -> io::Result<(IpAddr, Instant)> {
        let updated_now = Instant::now();
        let updated_long_ago = Instant::now() - *MAX_CACHE_AGE - Duration::from_secs(1);

        match Self::load_from_file(cache_file) {
            Ok(previously_cached_address) => {
                if Self::is_cache_file_old(cache_file) {
                    Ok((previously_cached_address, updated_long_ago))
                } else {
                    Ok((previously_cached_address, updated_now))
                }
            }
            Err(_) => {
                let fallback_address = Self::load_from_file(fallback_file)?;

                Ok((fallback_address, updated_long_ago))
            }
        }
    }

    fn load_from_file(file_path: &Path) -> io::Result<IpAddr> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut address = String::new();

        reader.read_line(&mut address)?;

        address
            .trim()
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid address data"))
    }

    fn is_cache_file_old(cache_file: &Path) -> bool {
        if let Ok(Some(cache_age)) = Self::read_cache_file_age(cache_file) {
            cache_age > *MAX_CACHE_AGE
        } else {
            true
        }
    }

    fn read_cache_file_age(cache_file: &Path) -> io::Result<Option<Duration>> {
        let metadata = cache_file.metadata()?;
        let last_modified = metadata.modified()?;

        Ok(last_modified.elapsed().ok())
    }

    fn resolve_into_cache(&mut self) {
        if let Ok(address) = self.dns_resolver.resolve(&self.hostname) {
            if self.cached_address != address {
                self.cached_address = address;
                self.update_cache_file();
            }

            self.last_updated = Instant::now();
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
    use std::io::{BufRead, BufReader, Write};
    use std::sync::atomic::{AtomicBool, Ordering};

    use self::filetime::FileTime;
    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_previously_cached_address() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address = "127.0.0.1".parse().unwrap();

        write_address(&cache_dir, cached_address);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert!(!mock_resolver.was_called());
        assert_eq!(address, cached_address);
    }

    #[test]
    fn old_cache_file_is_updated() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let cached_address = "127.0.0.1".parse().unwrap();

        let cache_file_path = write_address(&cache_dir, cached_address);

        make_file_old(&cache_file_path);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert_eq!(get_cached_address(&cache_dir), address.to_string());
        assert_eq!(address, mock_resolver.address());
    }

    #[test]
    fn old_cache_file_is_used_if_resolution_fails() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::that_fails();
        let cached_address = "127.0.0.1".parse().unwrap();

        let cache_file_path = write_address(&cache_dir, cached_address);

        make_file_old(&cache_file_path);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert_eq!(address, cached_address);
    }

    #[test]
    fn caches_resolved_ip() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let provided_address = "192.168.1.31".parse().unwrap();

        write_address(&fallback_dir, provided_address);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert_eq!(address, mock_resolver.address());
        assert_eq!(get_cached_address(&cache_dir), address.to_string());
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let (temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let provided_address = "192.168.1.31".parse().unwrap();

        write_address(&fallback_dir, provided_address);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);

        ::std::mem::drop(temp_dir);

        assert_eq!(cache.resolve(), mock_resolver.address());
    }

    #[test]
    fn uses_fallback_address() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let provided_address = "192.168.1.31".parse().unwrap();
        let mock_resolver = MockDnsResolver::that_fails();

        write_address(&fallback_dir, provided_address);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert!(mock_resolver.was_called());
        assert_eq!(address, provided_address);
    }

    #[test]
    fn ignores_fallback_address_if_resolution_succeeds() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");
        let provided_address = "192.168.1.31".parse().unwrap();

        write_address(&fallback_dir, provided_address);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert_eq!(address, mock_resolver.address());
    }

    #[test]
    fn doesnt_update_cache_file_if_resolved_address_is_the_same() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver = MockDnsResolver::from_str("192.168.1.206");

        let cache_file_path = write_address(&cache_dir, mock_resolver.address());
        let cache_file_last_updated = make_file_old(&cache_file_path);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert_eq!(address, mock_resolver.address());
        assert_eq!(
            get_file_last_updated(&cache_file_path),
            cache_file_last_updated
        );
    }

    fn create_test_dirs() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let fallback_dir = temp_dir.path().join("resource");

        fs::create_dir(&cache_dir).unwrap();
        fs::create_dir(&fallback_dir).unwrap();

        (temp_dir, cache_dir, fallback_dir)
    }

    fn write_address(dir: &Path, address: IpAddr) -> PathBuf {
        let file_path = dir.join("api_ip_address.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "{}", address).unwrap();

        file_path
    }

    fn make_file_old(file: &Path) -> FileTime {
        let file_metadata = file.metadata().unwrap();
        let last_access_time = FileTime::from_last_access_time(&file_metadata);
        let fake_modification_time = FileTime::from_seconds_since_1970(100_000, 0);

        filetime::set_file_times(&file, last_access_time, fake_modification_time).unwrap();

        fake_modification_time
    }

    fn get_cached_address(cache_dir: &Path) -> String {
        let cache_file_path = cache_dir.join("api_ip_address.txt");

        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();

        cache_reader.read_line(&mut cached_address).unwrap();

        cached_address.trim().to_string()
    }

    fn get_file_last_updated(file: &Path) -> FileTime {
        let file_metadata = file.metadata().unwrap();

        FileTime::from_last_modification_time(&file_metadata)
    }

    fn create_cached_dns_resolver<'a>(
        mock_resolver: &'a MockDnsResolver,
        cache_dir: &'a Path,
        fallback_dir: &'a Path,
    ) -> CachedDnsResolver<&'a MockDnsResolver> {
        let hostname = "dummy.host".to_owned();
        let filename = "api_ip_address.txt";

        CachedDnsResolver::with_dns_resolver(
            mock_resolver,
            hostname,
            cache_dir,
            fallback_dir,
            filename,
        ).expect("Failed to create CachedDnsResolver instance")
    }

    struct MockDnsResolver {
        address: Option<IpAddr>,
        called: AtomicBool,
    }

    impl MockDnsResolver {
        pub fn from_str(ip_address: &str) -> Self {
            MockDnsResolver {
                address: Some(ip_address.parse().unwrap()),
                called: AtomicBool::new(false),
            }
        }

        pub fn that_fails() -> Self {
            MockDnsResolver {
                address: None,
                called: AtomicBool::new(false),
            }
        }

        pub fn address(&self) -> IpAddr {
            self.address.unwrap()
        }

        pub fn was_called(&self) -> bool {
            self.called.load(Ordering::Acquire)
        }
    }

    impl<'r> DnsResolver for &'r MockDnsResolver {
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
