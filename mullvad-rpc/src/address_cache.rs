use std::{
    io,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
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

    #[error(display = "The address change listener returned an error")]
    ChangeListenerError,
}

pub type CurrentAddressChangeListener =
    dyn Fn(SocketAddr) -> Result<(), ()> + Send + Sync + 'static;

#[derive(Clone)]
pub struct AddressCache {
    inner: Arc<Mutex<AddressCacheInner>>,
    write_path: Option<Arc<Path>>,
    change_listener: Arc<Box<CurrentAddressChangeListener>>,
}

impl AddressCache {
    /// Initialize cache using the given list, and write changes to `write_path`.
    pub fn new(address: SocketAddr, write_path: Option<Box<Path>>) -> Result<Self, Error> {
        let cache = AddressCacheInner::from_address(address);
        log::trace!("API address cache: {:?}", cache.address);
        log::debug!("Using API address: {:?}", Self::get_address_inner(&cache));

        let address_cache = Self {
            inner: Arc::new(Mutex::new(cache)),
            write_path: write_path.map(|cache| Arc::from(cache)),
            change_listener: Arc::new(Box::new(|_| Ok(()))),
        };
        Ok(address_cache)
    }

    /// Initialize cache using `read_path`, and write changes to `write_path`.
    pub async fn from_file(read_path: &Path, write_path: Option<Box<Path>>) -> Result<Self, Error> {
        log::debug!("Loading API addresses from {}", read_path.display());
        Self::new(read_address_file(read_path).await?, write_path)
    }

    pub fn set_change_listener(&mut self, change_listener: Arc<Box<CurrentAddressChangeListener>>) {
        self.change_listener = change_listener;
    }

    /// Returns the currently selected address.
    pub fn get_address(&self) -> SocketAddr {
        let inner = self.inner.lock().unwrap();
        Self::get_address_inner(&inner)
    }

    fn get_address_inner(inner: &AddressCacheInner) -> SocketAddr {
        inner.address
    }

    pub async fn set_address(&self, address: SocketAddr) -> io::Result<()> {
        let should_update = {
            let mut inner = self.inner.lock().unwrap();
            let mut transaction = AddressCacheTransaction::new(&mut inner);

            let current_address = transaction.address.clone();

            if address != current_address {
                transaction.address = address.clone();
                tokio::task::block_in_place(move || {
                    if (*self.change_listener)(Self::get_address_inner(&transaction)).is_err() {
                        log::error!("Failed to select new API endpoint");
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "callback returned an error",
                        ));
                    }
                    transaction.commit();
                    Ok(())
                })?;
                true
            } else {
                false
            }
        };
        if should_update {
            log::trace!("API address cache: {}", address);
            self.save_to_disk().await?;
        }
        Ok(())
    }

    async fn save_to_disk(&self) -> io::Result<()> {
        let write_path = match self.write_path.as_ref() {
            Some(write_path) => write_path,
            None => return Ok(()),
        };

        let address = {
            let inner = self.inner.lock().unwrap();
            inner.address.clone()
        };

        let temp_path = write_path.with_file_name("api-cache.temp");

        let mut file = fs::File::create(&temp_path).await?;
        let mut contents = address.to_string();
        contents += "\n";
        file.write_all(contents.as_bytes()).await?;
        file.sync_data().await?;

        fs::rename(&temp_path, write_path).await
    }
}

impl crate::rest::AddressProvider for AddressCache {
    fn get_address(&self) -> String {
        self.get_address().to_string()
    }

    fn clone_box(&self) -> Box<dyn crate::rest::AddressProvider> {
        Box::new(self.clone())
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

struct AddressCacheTransaction<'a> {
    current: &'a mut AddressCacheInner,
    working_cache: AddressCacheInner,
}

impl<'a> AddressCacheTransaction<'a> {
    fn new(cache: &'a mut AddressCacheInner) -> Self {
        Self {
            working_cache: cache.clone(),
            current: cache,
        }
    }

    fn commit(self) {
        *self.current = self.working_cache;
    }
}

impl<'a> Deref for AddressCacheTransaction<'a> {
    type Target = AddressCacheInner;

    fn deref(&self) -> &Self::Target {
        &self.working_cache
    }
}

impl<'a> DerefMut for AddressCacheTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.working_cache
    }
}

async fn read_address_file(path: &Path) -> Result<SocketAddr, Error> {
    let mut file = fs::File::open(path)
        .await
        .map_err(|error| Error::OpenAddressCache(error))?;
    let mut address = String::new();
    file.read_to_string(&mut address)
        .await
        .map_err(Error::ReadAddressCache)?;
    address.trim().parse().map_err(|_| Error::ParseAddressCache)
}
