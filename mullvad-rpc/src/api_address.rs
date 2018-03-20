use std::fs::File;
use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;


pub static MASTER_API_HOST: &str = "api.mullvad.net";


pub trait DnsResolver {
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr>;
}

pub struct SystemDnsResolver;

impl DnsResolver for SystemDnsResolver {
    fn resolve(&mut self, host: &str) -> io::Result<IpAddr> {
        (host, 0)
            .to_socket_addrs()?
            .next()
            .map(|socket_address| socket_address.ip())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("could not resolve hostname {}", host),
                )
            })
    }
}


/// Returns the IP address of the Mullvad API server from cache if it exists, otherwise it tries to
/// use a predetermined address if exists or resolve it based on its hostname and cache the result.
pub fn api_address(cache_dir: Option<&Path>, resource_dir: Option<&Path>) -> String {
    get_api_address_using_resolver(SystemDnsResolver, cache_dir, resource_dir)
}

fn get_api_address_using_resolver<R: DnsResolver>(
    mut resolver: R,
    cache_dir: Option<&Path>,
    resource_dir: Option<&Path>,
) -> String {
    let cache_file = cache_dir.map(get_address_file_path);
    let provided_file = resource_dir.map(get_address_file_path);

    if let Ok(address) = read_address_from_file(cache_file.as_ref()) {
        address.to_string()
    } else {
        let resolved_address = read_address_from_file(provided_file.as_ref())
            .or_else(|_| resolver.resolve(MASTER_API_HOST));

        if let Ok(address) = resolved_address {
            let _ = store_address_in_cache(&address, cache_file.as_ref());
            address.to_string()
        } else {
            MASTER_API_HOST.to_string()
        }
    }
}

fn get_address_file_path(dir: &Path) -> PathBuf {
    dir.join("api_address.json")
}

fn store_address_in_cache(address: &IpAddr, cache_file: Option<&PathBuf>) -> Result<(), io::Error> {
    if let Some(cache_file) = cache_file {
        let file = File::create(cache_file)?;

        serde_json::to_writer(file, address)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "No cache file specified",
        ))
    }
}

fn read_address_from_file(file: Option<&PathBuf>) -> Result<IpAddr, io::Error> {
    if let Some(file) = file {
        let reader = File::open(file)?;
        let address = serde_json::from_reader(reader)?;

        Ok(address)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "No API IP address file",
        ))
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs::{create_dir, File};
    use std::io::{BufRead, BufReader, Write};

    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_cached_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_temp_dirs();
        let (_mock_address, mock_resolver) = create_mock_resolver();
        let cached_address = "127.0.0.1";

        {
            let cache_file_path = cache_dir.join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let address =
            get_api_address_using_resolver(mock_resolver, Some(&cache_dir), Some(&resource_dir));

        assert_eq!(address, cached_address);
    }

    #[test]
    fn uses_provided_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_temp_dirs();
        let (mock_address, mock_resolver) = create_mock_resolver();
        let provided_address = "192.168.1.98";

        assert_ne!(provided_address, mock_address.to_string());

        {
            let address_file_path = resource_dir.join("api_address.json");
            let mut address_file = File::create(address_file_path).unwrap();
            writeln!(address_file, "\"{}\"", provided_address).unwrap();
        }

        let address =
            get_api_address_using_resolver(mock_resolver, Some(&cache_dir), Some(&resource_dir));

        assert_eq!(address, provided_address);
        check_cached_address(&cache_dir, &provided_address);
    }

    #[test]
    fn caches_resolved_address() {
        let (_temp_dir, cache_dir, resource_dir) = create_temp_dirs();
        let (mock_address, mock_resolver) = create_mock_resolver();
        let address =
            get_api_address_using_resolver(mock_resolver, Some(&cache_dir), Some(&resource_dir));

        assert_eq!(address, mock_address.to_string());
        check_cached_address(&cache_dir, &mock_address.to_string());
    }

    #[test]
    fn resolves_even_if_cache_dir_is_unavailable() {
        let (temp_dir, cache_dir, resource_dir) = create_temp_dirs();
        let (mock_address, mock_resolver) = create_mock_resolver();

        ::std::mem::drop(temp_dir);

        assert_eq!(
            get_api_address_using_resolver(mock_resolver, Some(&cache_dir), Some(&resource_dir)),
            mock_address.to_string()
        );
    }

    fn create_temp_dirs() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new("address-cache-test").unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let resource_dir = temp_dir.path().join("resources");

        create_dir(&cache_dir).unwrap();
        create_dir(&resource_dir).unwrap();

        (temp_dir, cache_dir, resource_dir)
    }

    fn create_mock_resolver() -> (IpAddr, MockDnsResolver) {
        let mock_address = "192.168.1.196".parse().unwrap();
        let mock_resolver = MockDnsResolver {
            address: Some(mock_address),
        };

        (mock_address, mock_resolver)
    }

    fn check_cached_address(cache_dir: &Path, address: &str) {
        let cache_file_path = cache_dir.join("api_address.json");
        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();
        cache_reader.read_line(&mut cached_address).unwrap();

        assert_eq!(cached_address, format!("\"{}\"", address));
    }

    pub struct MockDnsResolver {
        address: Option<IpAddr>,
    }

    impl DnsResolver for MockDnsResolver {
        fn resolve(&mut self, host: &str) -> io::Result<IpAddr> {
            self.address.clone().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("hostname {:?} not found", host),
                )
            })
        }
    }
}
