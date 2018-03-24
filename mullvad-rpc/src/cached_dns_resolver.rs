use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::marker::PhantomData;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};


lazy_static! {
    static ref MAX_CACHE_AGE: Duration = Duration::from_secs(3600);
    static ref MAX_DNS_RESOLUTION_TIME: Duration = Duration::from_secs(5);
}


pub trait DnsResolver: Send + 'static {
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
    cache_file: PathBuf,
    cached_address: IpAddr,
    last_updated: Instant,
    hostname_tx: mpsc::Sender<String>,
    ip_address_rx: mpsc::Receiver<Option<IpAddr>>,
    dns_timeout: Duration,
    _dns_resolver: PhantomData<R>,
}

impl CachedDnsResolver<SystemDnsResolver> {
    pub fn new(
        hostname: String,
        cache_dir: &Path,
        fallback_dir: &Path,
        filename: &str,
    ) -> io::Result<Self> {
        Self::with_dns_resolver_and_dns_timeout(
            SystemDnsResolver,
            *MAX_DNS_RESOLUTION_TIME,
            hostname,
            cache_dir,
            fallback_dir,
            filename,
        )
    }
}

impl<R: DnsResolver> CachedDnsResolver<R> {
    pub fn with_dns_resolver_and_dns_timeout(
        dns_resolver: R,
        dns_timeout: Duration,
        hostname: String,
        cache_dir: &Path,
        fallback_dir: &Path,
        filename: &str,
    ) -> io::Result<Self> {
        let cache_file = cache_dir.join(filename);
        let fallback_file = fallback_dir.join(filename);

        let (cached_address, last_updated) =
            Self::load_initial_cached_address(&cache_file, &fallback_file)?;

        let (hostname_tx, hostname_rx) = mpsc::channel();
        let (ip_address_tx, ip_address_rx) = mpsc::channel();

        thread::spawn(move || {
            Self::resolution_worker_thread(hostname_rx, ip_address_tx, dns_resolver)
        });

        Ok(CachedDnsResolver {
            hostname,
            cache_file,
            cached_address,
            last_updated,
            hostname_tx,
            ip_address_rx,
            dns_timeout,
            _dns_resolver: PhantomData,
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
        if let Some(address) = self.resolve_using_dns_resolver() {
            if self.cached_address != address {
                self.cached_address = address;
                self.update_cache_file();
            }

            self.last_updated = Instant::now();
        }
    }

    fn resolve_using_dns_resolver(&mut self) -> Option<IpAddr> {
        self.hostname_tx.send(self.hostname.to_owned()).ok()?;
        self.ip_address_rx.recv_timeout(self.dns_timeout).ok()?
    }

    fn update_cache_file(&mut self) {
        if let Ok(mut cache_file) = File::create(&self.cache_file) {
            let _ = writeln!(cache_file, "{}", self.cached_address);
        }
    }

    fn resolution_worker_thread(
        requests: mpsc::Receiver<String>,
        ip_addresses: mpsc::Sender<Option<IpAddr>>,
        dns_resolver: R,
    ) {
        while let Ok(hostname) = requests.recv() {
            let ip_address = dns_resolver.resolve(&hostname);
            let send_result = ip_addresses.send(ip_address.ok());

            if send_result.is_err() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate filetime;
    extern crate tempdir;

    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Write};
    use std::sync::Arc;
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

    #[test]
    fn long_resolution_times_out() {
        let (_temp_dir, cache_dir, fallback_dir) = create_test_dirs();
        let mock_resolver =
            MockDnsResolver::from_str_with_delay("192.168.1.206", Duration::from_secs(10));
        let provided_address = "192.168.1.31".parse().unwrap();

        write_address(&fallback_dir, provided_address);

        let mut cache = create_cached_dns_resolver(&mock_resolver, &cache_dir, &fallback_dir);
        let address = cache.resolve();

        assert!(mock_resolver.was_called());
        assert_eq!(address, provided_address);
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

    fn create_cached_dns_resolver(
        mock_resolver: &Arc<MockDnsResolver>,
        cache_dir: &Path,
        fallback_dir: &Path,
    ) -> CachedDnsResolver<Arc<MockDnsResolver>> {
        let hostname = "dummy.host".to_owned();
        let filename = "api_ip_address.txt";
        let dns_timeout = Duration::from_millis(50);

        CachedDnsResolver::with_dns_resolver_and_dns_timeout(
            mock_resolver.clone(),
            dns_timeout,
            hostname,
            cache_dir,
            fallback_dir,
            filename,
        ).expect("Failed to create CachedDnsResolver instance")
    }

    struct MockDnsResolver {
        address: Option<IpAddr>,
        delay: Option<Duration>,
        called: AtomicBool,
    }

    impl MockDnsResolver {
        pub fn from_str(ip_address: &str) -> Arc<Self> {
            Arc::new(MockDnsResolver {
                address: Some(ip_address.parse().unwrap()),
                delay: None,
                called: AtomicBool::new(false),
            })
        }

        pub fn from_str_with_delay(ip_address: &str, delay: Duration) -> Arc<Self> {
            Arc::new(MockDnsResolver {
                address: Some(ip_address.parse().unwrap()),
                delay: Some(delay),
                called: AtomicBool::new(false),
            })
        }

        pub fn that_fails() -> Arc<Self> {
            Arc::new(MockDnsResolver {
                address: None,
                delay: None,
                called: AtomicBool::new(false),
            })
        }

        pub fn address(&self) -> IpAddr {
            self.address.unwrap()
        }

        pub fn was_called(&self) -> bool {
            self.called.load(Ordering::Acquire)
        }
    }

    impl DnsResolver for Arc<MockDnsResolver> {
        fn resolve(&self, host: &str) -> io::Result<IpAddr> {
            self.called.store(true, Ordering::Release);

            if let Some(delay) = self.delay {
                println!("Simulating long DNS resolution...");
                thread::sleep(delay);
            }

            self.address.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to resolve address for {:?}", host),
                )
            })
        }
    }
}
