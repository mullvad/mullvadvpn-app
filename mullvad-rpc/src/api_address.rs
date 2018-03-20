use std::fs::File;
use std::io;
use std::marker::PhantomData;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};

use serde_json;


pub static MASTER_API_HOST: &str = "api.mullvad.net";


/// Returns the IP address of the Mullvad API server from cache if it exists, otherwise it tries to
/// resolve it based on its hostname and cache the result.
pub fn api_address(resource_dir: Option<&Path>) -> String {
    ApiAddress::<(&'static str, u16)>::get(resource_dir)
}

struct ApiAddress<R>
where
    R: From<(&'static str, u16)> + ToSocketAddrs,
{
    _resolver: PhantomData<R>,
}

impl<R> ApiAddress<R>
where
    R: From<(&'static str, u16)> + ToSocketAddrs,
{
    fn get(resource_dir: Option<&Path>) -> String {
        if let Some(cache_file) = resource_dir.map(Self::get_cache_file_path) {
            if let Ok(address) = Self::read_cached_address(&cache_file) {
                address
            } else if let Ok(address) = Self::resolve_address_from_hostname() {
                let _ = Self::store_address_in_cache(&address, &cache_file);
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
        let resolver = R::from((MASTER_API_HOST, 0));

        resolver
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
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};
    use std::net::SocketAddr;
    use std::vec::IntoIter;

    use self::tempdir::TempDir;
    use super::*;

    #[test]
    fn uses_cached_address() {
        let temp_dir = TempDir::new("address-cache-test").unwrap();
        let cached_address = MockResolver::resolved_address().ip().to_string();

        {
            let cache_file_path = temp_dir.path().join("api_address.json");
            let mut cache_file = File::create(cache_file_path).unwrap();
            writeln!(cache_file, "\"{}\"", cached_address).unwrap();
        }

        let address = ApiAddress::<MockResolver>::get(Some(temp_dir.path()));

        assert_eq!(address, cached_address);
    }

    #[test]
    fn caches_resolved_address() {
        let temp_dir = TempDir::new("address-cache-test").unwrap();
        let address = ApiAddress::<MockResolver>::get(Some(temp_dir.path()));

        let cache_file_path = temp_dir.path().join("api_address.json");
        assert!(cache_file_path.exists());

        let cache_file = File::open(cache_file_path).unwrap();
        let mut cache_reader = BufReader::new(cache_file);
        let mut cached_address = String::new();
        cache_reader.read_line(&mut cached_address).unwrap();

        assert_eq!(address, MockResolver::resolved_address().ip().to_string());
        assert_eq!(cached_address, format!("\"{}\"", address));
    }

    #[test]
    fn resolves_even_if_cache_dir_is_unavailable() {
        let temp_dir = TempDir::new("address-cache-test").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        ::std::mem::drop(temp_dir);

        assert_eq!(
            ApiAddress::<MockResolver>::get(Some(&temp_dir_path)),
            MockResolver::resolved_address().ip().to_string(),
        );
    }

    struct MockResolver {
        resolved_address: SocketAddr,
    }

    impl MockResolver {
        pub fn resolved_address() -> SocketAddr {
            "127.0.0.1:48719".parse().unwrap()
        }
    }

    impl From<(&'static str, u16)> for MockResolver {
        fn from(_: (&'static str, u16)) -> Self {
            MockResolver {
                resolved_address: MockResolver::resolved_address(),
            }
        }
    }

    impl ToSocketAddrs for MockResolver {
        type Iter = IntoIter<SocketAddr>;

        fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
            Ok(vec![self.resolved_address.clone()].into_iter())
        }
    }
}
