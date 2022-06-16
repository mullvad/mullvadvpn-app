use super::API;
use std::{io, net::SocketAddr, path::Path, sync::Arc};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open the address cache file")]
    OpenAddressCache(#[error(source)] io::Error),

    #[error(display = "Failed to read the address cache file")]
    ReadAddressCache(#[error(source)] io::Error),

    #[error(display = "Failed to parse the address cache file")]
    ParseAddressCache,

    #[error(display = "Failed to update the address cache file")]
    WriteAddressCache(#[error(source)] io::Error),

    #[error(display = "The address cache is empty")]
    EmptyAddressCache,
}

#[derive(Clone)]
pub struct AddressCache {
    inner: Arc<Mutex<AddressCacheInner>>,
    write_path: Option<Arc<Path>>,
}

impl AddressCache {
    /// Initialize cache using the hardcoded address, and write changes to `write_path`.
    pub fn new(write_path: Option<Box<Path>>) -> Result<Self, Error> {
        Self::new_inner(API.addr, write_path)
    }

    /// Initialize cache using `read_path`, and write changes to `write_path`.
    pub async fn from_file(read_path: &Path, write_path: Option<Box<Path>>) -> Result<Self, Error> {
        log::debug!("Loading API addresses from {}", read_path.display());
        Self::new_inner(read_address_file(read_path).await?, write_path)
    }

    fn new_inner(address: SocketAddr, write_path: Option<Box<Path>>) -> Result<Self, Error> {
        let cache = AddressCacheInner::from_address(address);
        log::debug!("Using API address: {}", cache.address);

        let address_cache = Self {
            inner: Arc::new(Mutex::new(cache)),
            write_path: write_path.map(Arc::from),
        };
        Ok(address_cache)
    }

    /// Returns the address if the hostname equals `API.host`. Otherwise, returns `None`.
    pub async fn resolve_hostname(&self, hostname: &str) -> Option<SocketAddr> {
        if hostname.eq_ignore_ascii_case(&API.host) {
            Some(self.get_address().await)
        } else {
            None
        }
    }

    /// Returns the currently selected address.
    pub async fn get_address(&self) -> SocketAddr {
        self.inner.lock().await.address
    }

    pub async fn set_address(&self, address: SocketAddr) -> io::Result<()> {
        let mut inner = self.inner.lock().await;
        if address != inner.address {
            self.save_to_disk(&address).await?;
            inner.address = address;
        }
        Ok(())
    }

    async fn save_to_disk(&self, address: &SocketAddr) -> io::Result<()> {
        let write_path = match self.write_path.as_ref() {
            Some(write_path) => write_path,
            None => return Ok(()),
        };

        let mut file = crate::fs::AtomicFile::new(write_path.to_path_buf()).await?;
        let mut contents = address.to_string();
        contents += "\n";
        file.write_all(contents.as_bytes()).await?;
        file.finalize().await
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

async fn read_address_file(path: &Path) -> Result<SocketAddr, Error> {
    let mut file = fs::File::open(path)
        .await
        .map_err(Error::OpenAddressCache)?;
    let mut address = String::new();
    file.read_to_string(&mut address)
        .await
        .map_err(Error::ReadAddressCache)?;
    address.trim().parse().map_err(|_| Error::ParseAddressCache)
}
