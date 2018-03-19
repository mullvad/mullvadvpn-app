use std::fs::File;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;

use super::MASTER_API_HOST;

pub struct AddressCache {
    cache_file: PathBuf,
}

impl AddressCache {
    pub fn new(cache_dir: &Path) -> Self {
        AddressCache {
            cache_file: cache_dir.join("api_address.json"),
        }
    }

    pub fn api_address(&self) -> Option<String> {
        self.load_from_cache()
            .or_else(|_| self.resolve_into_cache())
            .map(|address| address.to_string())
            .ok()
    }

    fn load_from_cache(&self) -> Result<SocketAddr, io::Error> {
        let cache_file = File::open(&self.cache_file)?;
        let address = serde_json::from_reader(cache_file)?;

        Ok(address)
    }

    fn resolve_into_cache(&self) -> Result<SocketAddr, io::Error> {
        let address = Self::resolve_address()?.into();

        let _ = self.store_in_cache(&address);

        Ok(address)
    }

    fn resolve_address() -> Result<SocketAddr, io::Error> {
        (MASTER_API_HOST, 0)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Mullvad RPC API host not found")
            })
    }

    fn store_in_cache(&self, address: &SocketAddr) -> Result<(), io::Error> {
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
        let cached_address = SocketAddr::new("127.0.0.1".parse().unwrap(), 52780);

        {
            let cache_file_path = temp_dir.path().join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let cache = AddressCache::new(temp_dir.path());
        let address = cache.api_address().unwrap();

        assert_eq!(
            address,
            format!("{}:{}", cached_address.ip(), cached_address.port())
        );
    }

    #[test]
    fn caches_resolved_ip() {
        let temp_dir = TempDir::new("ip-cache-test").unwrap();
        let cache = AddressCache::new(temp_dir.path());
        let address = cache.api_address().unwrap();

        let cache_file_path = temp_dir.path().join("api_address.json");
        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();
        cache_reader.read_line(&mut cached_address).unwrap();

        assert_eq!(cached_address, format!("\"{}\"", address));
    }

    #[test]
    fn resolves_even_if_impossible_to_store_in_cache() {
        let temp_dir_path = TempDir::new("ip-cache-test").unwrap().path().to_path_buf();
        let cache = AddressCache::new(&temp_dir_path);

        assert!(cache.api_address().is_some());
    }
}
