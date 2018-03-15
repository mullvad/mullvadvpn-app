use std::fs::File;
use std::io;
use std::net::{AddrParseError, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;


pub static MASTER_API_HOST: &str = "api.mullvad.net";


#[derive(Clone, Debug, Deserialize, Serialize)]
struct IpAddress {
    ip_address: String,
    port: u16,
}

impl IpAddress {
    pub fn try_into_socket_addr(self) -> Result<SocketAddr, AddrParseError> {
        let ip_address = self.ip_address.parse()?;

        Ok(SocketAddr::new(ip_address, self.port))
    }
}

/// Returns the IP address of the Mullvad API server from cache if it exists, otherwise it tries to
/// resolve it based on its hostname and cache the result.
pub fn api_ip_address(resource_dir: &Path) -> String {
    let cache_file = get_cache_file_path(resource_dir);
    if let Ok(ip_address) = read_cached_ip_address(&cache_file) {
        ip_address
    } else if let Ok(ip_address) = resolve_ip_address_from_hostname() {
        let _ = store_ip_address_in_cache(&ip_address, &cache_file);
        ip_address.to_string()
    } else {
        MASTER_API_HOST.to_string()
    }
}

fn get_cache_file_path(resource_dir: &Path) -> PathBuf {
    resource_dir.join("api_ip_address.json")
}

fn read_cached_ip_address(cache_file: &Path) -> Result<String, io::Error> {
    let reader = File::open(cache_file)?;
    let cached_address: IpAddress = serde_json::from_reader(reader)?;

    cached_address
        .try_into_socket_addr()
        .map(|address| address.to_string())
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
}

fn resolve_ip_address_from_hostname() -> Result<SocketAddr, io::Error> {
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

fn store_ip_address_in_cache(address: &SocketAddr, cache_file: &Path) -> Result<(), io::Error> {
    let file = File::create(cache_file)?;
    let address_to_cache = IpAddress {
        ip_address: address.ip().to_string(),
        port: address.port(),
    };
    serde_json::to_writer(file, &address_to_cache)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
}
