use std::fs::File;
use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;


pub static MASTER_API_HOST: &str = "api.mullvad.net";


/// Returns the IP address of the Mullvad API server from cache if it exists, otherwise it tries to
/// resolve it based on its hostname and cache the result.
pub fn api_address(resource_dir: Option<&Path>) -> String {
    if let Some(cache_file) = resource_dir.map(get_cache_file_path) {
        if let Ok(address) = read_cached_address(&cache_file) {
            address
        } else if let Ok(address) = resolve_address_from_hostname() {
            let _ = store_address_in_cache(&address, &cache_file);
            address.to_string()
        } else {
            MASTER_API_HOST.to_string()
        }
    } else {
        MASTER_API_HOST.to_string()
    }
}

fn get_cache_file_path(resource_dir: &Path) -> PathBuf {
    resource_dir.join("api_address.json")
}

fn read_cached_address(cache_file: &Path) -> Result<String, io::Error> {
    let reader = File::open(cache_file)?;
    let address: IpAddr = serde_json::from_reader(reader)?;

    Ok(address.to_string())
}

fn resolve_address_from_hostname() -> Result<IpAddr, io::Error> {
    (MASTER_API_HOST, 0)
        .to_socket_addrs()?
        .next()
        .map(|socket_address| socket_address.ip())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("could not resolve hostname {}", MASTER_API_HOST),
            )
        })
}

fn store_address_in_cache(address: &IpAddr, cache_file: &Path) -> Result<(), io::Error> {
    let file = File::create(cache_file)?;
    serde_json::to_writer(file, address)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
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
        let temp_dir = TempDir::new("address-cache-test").unwrap();
        let cached_address = "127.0.0.1";

        {
            let cache_file_path = temp_dir.path().join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let address = api_address(Some(temp_dir.path()));

        assert_eq!(address, cached_address);
    }

    #[test]
    fn caches_resolved_address() {
        let temp_dir = TempDir::new("address-cache-test").unwrap();
        let address = api_address(Some(temp_dir.path()));

        let cache_file_path = temp_dir.path().join("api_address.json");
        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();
        cache_reader.read_line(&mut cached_address).unwrap();

        assert_eq!(cached_address, format!("\"{}\"", address));
    }

    #[test]
    fn resolves_even_if_cache_dir_is_unavailble() {
        let temp_dir_path = TempDir::new("address-cache-test")
            .unwrap()
            .path()
            .to_path_buf();

        assert_ne!(api_address(Some(&temp_dir_path)), MASTER_API_HOST);
    }
}
