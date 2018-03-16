use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;

use super::MASTER_API_HOST;

#[derive(Deserialize, Serialize)]
struct AddressCacheData {
    ip: String,
    port: u16,
}

impl AddressCacheData {
    fn is_valid(&self) -> bool {
        self.ip.parse::<IpAddr>().is_ok()
    }
}

impl From<SocketAddr> for AddressCacheData {
    fn from(address: SocketAddr) -> Self {
        AddressCacheData {
            ip: address.ip().to_string(),
            port: address.port(),
        }
    }
}

impl Display for AddressCacheData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.ip, self.port)
    }
}

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

    fn load_from_cache(&self) -> Result<AddressCacheData, io::Error> {
        let cache_file = File::open(&self.cache_file)?;
        let address: AddressCacheData = serde_json::from_reader(cache_file)?;

        if address.is_valid() {
            Ok(address)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "cached address is invalid",
            ))
        }
    }

    fn resolve_into_cache(&self) -> Result<AddressCacheData, io::Error> {
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

    fn store_in_cache(&self, address: &AddressCacheData) -> Result<(), io::Error> {
        let cache_file = File::create(&self.cache_file)?;

        serde_json::to_writer(&cache_file, &address)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }
}
