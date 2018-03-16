use std::fs::File;
use std::io;
use std::net::{AddrParseError, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;


pub static MASTER_API_HOST: &str = "api.mullvad.net";


#[derive(Clone, Debug, Deserialize, Serialize)]
struct CachedAddress {
    ip: String,
    port: u16,
}

impl CachedAddress {
    pub fn try_into_socket_addr(self) -> Result<SocketAddr, AddrParseError> {
        let ip = self.ip.parse()?;

        Ok(SocketAddr::new(ip, self.port))
    }
}

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
    let cached_address: CachedAddress = serde_json::from_reader(reader)?;

    cached_address
        .try_into_socket_addr()
        .map(|address| address.to_string())
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
}

fn resolve_address_from_hostname() -> Result<SocketAddr, io::Error> {
    (MASTER_API_HOST, 0)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("could not resolve hostname {}", MASTER_API_HOST),
            )
        })
}

fn store_address_in_cache(address: &SocketAddr, cache_file: &Path) -> Result<(), io::Error> {
    let file = File::create(cache_file)?;
    let address_to_cache = CachedAddress {
        ip: address.ip().to_string(),
        port: address.port(),
    };
    serde_json::to_writer(file, &address_to_cache)
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
        let cached_address = CachedAddress {
            ip: "127.0.0.1".to_string(),
            port: 52780,
        };

        {
            let cache_file_path = temp_dir.path().join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(
                cache_file,
                "{{ \"ip\": \"{}\", \"port\": {} }}",
                cached_address.ip, cached_address.port
            ).unwrap();
        }

        let address = api_address(Some(temp_dir.path()));

        assert_eq!(
            address,
            format!("{}:{}", cached_address.ip, cached_address.port)
        );
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

        let mut address_parts = address.split(":");
        let ip = address_parts.next().unwrap();
        let port = address_parts.next().unwrap();
        assert!(address_parts.next().is_none());

        assert_eq!(
            cached_address,
            format!("{{\"ip\":\"{}\",\"port\":{}}}", ip, port)
        );
    }
}
