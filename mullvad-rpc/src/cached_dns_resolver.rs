use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

pub struct CachedDnsResolver {
    hostname: String,
    cache_file: PathBuf,
    cached_address: Option<IpAddr>,
}

impl CachedDnsResolver {
    pub fn new(hostname: String, cache_file: PathBuf) -> Self {
        let cached_address = Self::load_from_file(&cache_file).ok();

        CachedDnsResolver {
            hostname,
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
        let address = Self::resolve_address(&self.hostname)?;

        let _ = self.store_in_cache(address);

        Ok(address)
    }

    fn resolve_address(hostname: &str) -> io::Result<IpAddr> {
        (hostname, 0)
            .to_socket_addrs()?
            .next()
            .map(|socket_address| socket_address.ip())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Mullvad RPC API host not found")
            })
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

    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_cached_address() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let cached_address = "127.0.0.1".parse().unwrap();

        write_address(&cache_dir, cached_address);

        let cache = create_cached_dns_resolver(&cache_dir);
        let address = cache.resolve().unwrap();

        assert_eq!(address, cached_address);
    }

    #[test]
    fn caches_resolved_ip() {
        let (_temp_dir, cache_dir) = create_test_dirs();
        let cache = create_cached_dns_resolver(&cache_dir);

        let address = cache.resolve().unwrap();

        assert_eq!(get_cached_address(&cache_dir), address.to_string());
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let (temp_dir, cache_dir) = create_test_dirs();
        let cache = create_cached_dns_resolver(&cache_dir);

        ::std::mem::drop(temp_dir);

        assert!(cache.resolve().is_some());
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

    fn create_cached_dns_resolver(cache_dir: &Path) -> CachedDnsResolver {
        let hostname = "api.mullvad.net".to_owned();
        let filename = "api_ip_address.txt";
        let cache_file = cache_dir.join(filename);

        CachedDnsResolver::new(hostname, cache_file)
    }
}
