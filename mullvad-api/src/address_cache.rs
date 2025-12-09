//! This module keeps track of the last known good API IP address and reads and stores it on disk.

use crate::{ApiEndpoint, DnsResolver};
use async_trait::async_trait;
use std::{io, net::SocketAddr, path::Path, sync::Arc};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Attempt to read without a path specified")]
    NoPath,

    #[error("Failed to open the address cache file")]
    Open(#[source] io::Error),

    #[error("Failed to read the address cache file")]
    Read(#[source] io::Error),

    #[error("Failed to parse the address cache file")]
    Parse,

    #[error("Failed to update the address cache file")]
    Write(#[source] io::Error),
}

/// a backing store for an AddressCache.

#[async_trait]
pub trait AddressCacheBacking: Sync {
    async fn read(&self) -> Result<String, Error>;
    async fn write(&self, data: &[u8]) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct FileAddressCacheBacking {
    read_path: Option<Arc<Path>>,
    write_path: Option<Arc<Path>>,
}

#[async_trait]
impl AddressCacheBacking for FileAddressCacheBacking {
    async fn read(&self) -> Result<String, Error> {
        let read_path = match self.read_path.as_ref() {
            Some(read_path) => read_path,
            None => return Err(Error::NoPath),
        };
        let mut file = fs::File::open(read_path).await.map_err(Error::Open)?;
        let mut result = String::new();
        file.read_to_string(&mut result)
            .await
            .map_err(Error::Read)?;
        Ok(result)
    }

    async fn write(&self, data: &[u8]) -> Result<(), Error> {
        let write_path = match self.write_path.as_ref() {
            Some(write_path) => write_path,
            None => return Ok(()),
        };
        let mut file = mullvad_fs::AtomicFile::new(&**write_path)
            .await
            .map_err(Error::Open)?;
        file.write_all(data).await.map_err(Error::Write)?;
        file.finalize().await.map_err(Error::Write)
    }
}

/// A DNS resolver which resolves using `AddressCache`.
#[async_trait]
impl DnsResolver for GenericAddressCache {
    async fn resolve(&self, host: String) -> Result<Vec<SocketAddr>, io::Error> {
        self.resolve_hostname(&host)
            .await
            .map(|addr| vec![addr])
            .ok_or(io::Error::other("host does not match API host"))
    }
}

#[derive(Clone)]
pub struct GenericAddressCache<Backing: AddressCacheBacking = FileAddressCacheBacking> {
    hostname: String,
    inner: Arc<Mutex<AddressCacheInner>>,
    backing: Backing,
}

pub type AddressCache = GenericAddressCache<FileAddressCacheBacking>;

impl<Backing: AddressCacheBacking> GenericAddressCache<Backing> {
    /// Initialise cache using a hardcoded address and a Backing for writing to
    pub fn new_with_address(endpoint: &ApiEndpoint, backing: Backing) -> Self {
        Self::new_inner(endpoint.address(), endpoint.host().to_owned(), backing)
    }

    /// Initialize cache using the hardcoded address, and write changes to `write_path`.
    pub fn new(endpoint: &ApiEndpoint, write_path: Option<Box<Path>>) -> AddressCache {
        AddressCache::new_with_address(
            endpoint,
            FileAddressCacheBacking {
                read_path: None,
                write_path: write_path.map(Arc::from),
            },
        )
    }

    pub async fn from_backing(hostname: String, backing: Backing) -> Result<Self, Error> {
        let address = read_address_backing(&backing).await?;
        Ok(Self::new_inner(address, hostname, backing))
    }

    /// Initialize cache using `read_path`, and write changes to `write_path`.
    pub async fn from_file(
        read_path: &Path,
        write_path: Option<Box<Path>>,
        hostname: String,
    ) -> Result<AddressCache, Error> {
        log::debug!("Loading API addresses from {}", read_path.display());
        AddressCache::from_backing(
            hostname,
            FileAddressCacheBacking {
                read_path: Some(Arc::from(read_path)),
                write_path: write_path.map(Arc::from),
            },
        )
        .await
    }

    fn new_inner(address: SocketAddr, hostname: String, backing: Backing) -> Self {
        let cache = AddressCacheInner::from_address(address);
        log::debug!("Using API address: {}", cache.address);

        Self {
            inner: Arc::new(Mutex::new(cache)),
            hostname,
            backing,
        }
    }

    /// Returns the address if the hostname equals `API.host`. Otherwise, returns `None`.
    async fn resolve_hostname(&self, hostname: &str) -> Option<SocketAddr> {
        if hostname.eq_ignore_ascii_case(&self.hostname) {
            Some(self.get_address().await)
        } else {
            None
        }
    }

    /// Returns the currently selected address.
    pub async fn get_address(&self) -> SocketAddr {
        self.inner.lock().await.address
    }

    pub async fn set_address(&self, address: SocketAddr) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        if address != inner.address {
            self.save_to_backing(&address).await?;
            inner.address = address;
        }
        Ok(())
    }

    async fn save_to_backing(&self, address: &SocketAddr) -> Result<(), Error> {
        let mut contents = address.to_string();
        contents += "\n";
        self.backing.write(contents.as_bytes()).await
    }
}

#[derive(Clone, PartialEq, Eq)]
struct AddressCacheInner {
    address: SocketAddr,
}

impl AddressCacheInner {
    fn from_address(address: SocketAddr) -> Self {
        Self { address }
    }
}

async fn read_address_backing<T: AddressCacheBacking>(backing: &T) -> Result<SocketAddr, Error> {
    backing
        .read()
        .await
        .and_then(|a| a.trim().parse().map_err(|_| Error::Parse))
}
