use log::{debug, info, warn};
use std::{
    fs::{self, File},
    io::{self, Write},
    net::{IpAddr, ToSocketAddrs},
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use talpid_types::ErrorExt;


static DNS_TIMEOUT: Duration = Duration::from_secs(2);
static MAX_CACHE_AGE: Duration = Duration::from_secs(3600);
static EXPIRED_CACHE_TIMESTAMP: SystemTime = UNIX_EPOCH;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// DNS resolution for a host took too long
    #[error(display = "DNS resolution for \"{}\" timed out", _0)]
    DnsTimeout(String, #[error(source)] mpsc::RecvTimeoutError),

    /// DNS resolution for a host didn't return any IP addresses
    #[error(display = "DNS resolution for \"{}\" did not return any IPs", _0)]
    HostNotFound(String),

    /// Failed to resolve IP address for host
    #[error(display = "Failed to resolve IP address for \"{}\"", _0)]
    ResolveFailure(String, #[error(source)] io::Error),

    /// Unable to read IP cache file
    #[error(display = "Failed to read DNS IP cache file")]
    ReadCacheError(#[error(source)] io::Error),

    /// Address loaded from file is invalid
    #[error(display = "Address loaded from file is invalid")]
    ParseCacheError(#[error(source)] std::net::AddrParseError),
}


pub trait DnsResolver {
    fn resolve(&mut self, host: &str) -> Result<IpAddr>;
}

pub struct SystemDnsResolver;

impl SystemDnsResolver {
    fn resolve_in_background_thread(host: &str) -> mpsc::Receiver<Result<IpAddr>> {
        let host = host.to_owned();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let _ = tx.send(Self::resolve_hostname(&host));
        });

        rx
    }

    fn resolve_hostname(host: &str) -> Result<IpAddr> {
        (host, 0)
            .to_socket_addrs()
            .map_err(|e| Error::ResolveFailure(host.to_owned(), e))?
            .next()
            .map(|socket_address| socket_address.ip())
            .ok_or_else(|| Error::HostNotFound(host.to_owned()))
    }
}

impl DnsResolver for SystemDnsResolver {
    fn resolve(&mut self, host: &str) -> Result<IpAddr> {
        Self::resolve_in_background_thread(host)
            .recv_timeout(DNS_TIMEOUT)
            .map_err(|e| Error::DnsTimeout(host.to_owned(), e))
            .and_then(|result| result)
    }
}

pub struct CachedDnsResolver<R: DnsResolver = SystemDnsResolver> {
    hostname: String,
    dns_resolver: R,
    cache_file: Option<PathBuf>,
    cached_address: IpAddr,
    last_updated: SystemTime,
}

impl CachedDnsResolver<SystemDnsResolver> {
    pub fn new(hostname: String, cache_file: Option<PathBuf>, fallback_address: IpAddr) -> Self {
        Self::with_dns_resolver(SystemDnsResolver, hostname, cache_file, fallback_address)
    }
}

impl<R: DnsResolver> CachedDnsResolver<R> {
    pub fn with_dns_resolver(
        dns_resolver: R,
        hostname: String,
        cache_file: Option<PathBuf>,
        fallback_address: IpAddr,
    ) -> Self {
        let (cached_address, last_updated) = match &cache_file {
            Some(cache_file) => Self::load_initial_cached_address(&cache_file, fallback_address),
            None => (fallback_address, EXPIRED_CACHE_TIMESTAMP),
        };

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
            warn!("System time changed, assuming cached IP address has expired");
            self.resolve_into_cache();
        }

        self.cached_address
    }

    fn load_initial_cached_address(
        cache_file: &Path,
        fallback_address: IpAddr,
    ) -> (IpAddr, SystemTime) {
        match Self::load_from_file(cache_file) {
            Ok(previously_cached_address) => match Self::read_file_modification_time(cache_file) {
                Ok(last_updated) => (previously_cached_address, last_updated),
                Err(error) => {
                    warn!("Failed to read modification time of file: {}", error);
                    (previously_cached_address, EXPIRED_CACHE_TIMESTAMP)
                }
            },
            Err(error) => {
                info!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to load previously cached IP address, using fallback"
                    )
                );
                (fallback_address, EXPIRED_CACHE_TIMESTAMP)
            }
        }
    }

    fn load_from_file(file_path: &Path) -> Result<IpAddr> {
        let address = fs::read_to_string(file_path).map_err(Error::ReadCacheError)?;
        address.trim().parse().map_err(Error::ParseCacheError)
    }

    fn read_file_modification_time(cache_file: &Path) -> io::Result<SystemTime> {
        cache_file
            .metadata()
            .and_then(|metadata| metadata.modified())
    }

    fn resolve_into_cache(&mut self) {
        debug!("Resolving IP for {}", self.hostname);
        match self.dns_resolver.resolve(&self.hostname) {
            Ok(address) => {
                if Self::is_bogus_address(address) {
                    warn!(
                        "DNS lookup for {} returned bogus address {}, ignoring",
                        self.hostname, address
                    );
                    return;
                }

                debug!("Updating DNS cache for {} with {}", self.hostname, address);
                self.cached_address = address;
                self.last_updated = SystemTime::now();

                if let Err(error) = self.update_cache_file() {
                    warn!("Failed to update cache file with new IP address: {}", error);
                }
            }
            Err(e) => {
                warn!(
                    "{}",
                    e.display_chain_with_msg(&format!("Unable to resolve {}", self.hostname))
                );
            }
        }
    }

    /// Checks if an IP seems to be a reasonable and routable IP. Used to try to filter out and
    /// ignore invalid IPs returned by poisoned DNS etc.
    fn is_bogus_address(address: IpAddr) -> bool {
        let is_private = match address {
            IpAddr::V4(address) => address.is_private(),
            _ => false,
        };
        address.is_unspecified() || address.is_loopback() || is_private
    }

    fn update_cache_file(&mut self) -> io::Result<()> {
        if let Some(cache_file_path) = &self.cache_file {
            let mut cache_file = File::create(cache_file_path)?;
            writeln!(cache_file, "{}", self.cached_address)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::{Read, Write},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    use super::*;
    use filetime::FileTime;
    use tempfile::TempDir;

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
        let cached_address = "80.10.20.30".parse().unwrap();
        let mock_address = "90.168.1.206".parse().unwrap();
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
        let mock_address = "80.10.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);
        let address = cache.resolve();

        assert_eq!(address, mock_address);
        assert_eq!(get_cached_address(&cache_dir), address.to_string());
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let (temp_dir, cache_dir) = create_test_dirs();
        let mock_address = "201.0.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache = create_cached_dns_resolver(mock_resolver, &cache_dir, None);

        std::mem::drop(temp_dir);

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
        let fallback_address = "200.10.1.31".parse().unwrap();
        let mock_address = "150.10.1.206".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache =
            create_cached_dns_resolver(mock_resolver, &cache_dir, Some(fallback_address));
        let address = cache.resolve();

        assert_eq!(address, mock_address);
    }

    #[test]
    fn invalid_cache_file_leads_to_fallback_address_usage() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let fallback_address = "160.20.1.31".parse().unwrap();
        let mock_resolver = MockDnsResolver::that_fails();
        let mock_resolver_was_called = mock_resolver.was_called_handle();

        write_invalid_address(&cache_dir);

        let mut cache =
            create_cached_dns_resolver(mock_resolver, &cache_dir, Some(fallback_address));
        let address = cache.resolve();

        assert!(mock_resolver_was_called.load(Ordering::Acquire));
        assert_eq!(address, fallback_address);
    }

    #[test]
    fn ignores_private_ip() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let fallback_address = "160.20.1.31".parse().unwrap();
        let mock_address = "10.100.200.1".parse().unwrap();
        let mock_resolver = MockDnsResolver::with_address(mock_address);

        let mut cache =
            create_cached_dns_resolver(mock_resolver, &cache_dir, Some(fallback_address));
        let address = cache.resolve();

        assert_eq!(address, fallback_address);
        let cache_file_path = cache_dir.join(crate::API_IP_CACHE_FILENAME);
        assert!(!cache_file_path.exists());
    }

    fn create_test_dirs() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create a temporary cache directory");
        let cache_dir = temp_dir.path().join("cache");

        fs::create_dir(&cache_dir).unwrap();

        (temp_dir, cache_dir)
    }

    fn write_invalid_address(dir: &Path) -> PathBuf {
        let file_path = dir.join(crate::API_IP_CACHE_FILENAME);
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "400.30.12.9").unwrap();

        file_path
    }

    fn write_address(dir: &Path, address: IpAddr) -> PathBuf {
        let file_path = dir.join(crate::API_IP_CACHE_FILENAME);
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "{}", address).unwrap();

        file_path
    }

    fn make_file_old(file: &Path) {
        let file_metadata = file.metadata().unwrap();
        let last_access_time = FileTime::from_last_access_time(&file_metadata);
        let fake_modification_time = FileTime::from_unix_time(100_000, 0);

        filetime::set_file_times(&file, last_access_time, fake_modification_time).unwrap();
    }

    fn get_cached_address(cache_dir: &Path) -> String {
        let cache_file_path = cache_dir.join(crate::API_IP_CACHE_FILENAME);

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
        let hostname = String::from("dummy.host");
        let cache_file = cache_dir.join(crate::API_IP_CACHE_FILENAME);
        let fallback_address = fallback_address.unwrap_or(IpAddr::from([10, 0, 109, 91]));

        CachedDnsResolver::with_dns_resolver(
            mock_resolver,
            hostname,
            Some(cache_file),
            fallback_address,
        )
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
        fn resolve(&mut self, host: &str) -> Result<IpAddr> {
            self.called.store(true, Ordering::Release);
            self.address.ok_or_else(|| {
                Error::ResolveFailure(
                    host.to_owned(),
                    io::Error::new(io::ErrorKind::Other, "FAILED"),
                )
            })
        }
    }
}
