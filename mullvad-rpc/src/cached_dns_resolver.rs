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
