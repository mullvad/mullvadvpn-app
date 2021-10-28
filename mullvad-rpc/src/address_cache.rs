use super::API_ADDRESS;
use rand::seq::SliceRandom;
use std::{
    io,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    path::Path,
    sync::{Arc, Mutex},
};
use talpid_types::ErrorExt;
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open the address cache file")]
    OpenAddressCache(#[error(source)] io::Error),

    #[error(display = "Failed to read the address cache file")]
    ReadAddressCache(#[error(source)] io::Error),

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
    pub fn new(
        addresses: Vec<SocketAddr>,
        write_path: Option<Box<Path>>,
        change_listener: Arc<Box<CurrentAddressChangeListener>>,
    ) -> Result<Self, Error> {
        let mut cache = AddressCacheInner::from_addresses(addresses)?;
        cache.shuffle_tail();
        log::trace!("API address cache: {:?}", cache.addresses);
        log::debug!("Using API address: {:?}", Self::get_address_inner(&cache));

        let address_cache = Self {
            inner: Arc::new(Mutex::new(cache)),
            write_path: write_path.map(|cache| Arc::from(cache)),
            change_listener,
        };
        Ok(address_cache)
    }

    /// Initialize cache using `read_path`, and write changes to `write_path`.
    pub async fn from_file(
        read_path: &Path,
        write_path: Option<Box<Path>>,
        change_listener: Arc<Box<CurrentAddressChangeListener>>,
    ) -> Result<Self, Error> {
        log::debug!("Loading API addresses from {:?}", read_path);
        Self::new(
            read_address_file(read_path).await?,
            write_path,
            change_listener,
        )
    }

    pub fn set_change_listener(&mut self, change_listener: Arc<Box<CurrentAddressChangeListener>>) {
        self.change_listener = change_listener;
    }

    /// Returns the currently selected address.
    pub fn get_address(&self) -> SocketAddr {
        let mut inner = self.inner.lock().unwrap();
        inner.tried_current = true;
        Self::get_address_inner(&inner)
    }

    /// Returns the current address without registering it as "tried"
    /// in [`has_tried_current_address`].
    pub fn peek_address(&self) -> SocketAddr {
        let inner = self.inner.lock().unwrap();
        Self::get_address_inner(&inner)
    }

    fn get_address_inner(inner: &AddressCacheInner) -> SocketAddr {
        if inner.addresses.is_empty() {
            return *API_ADDRESS;
        }
        *inner
            .addresses
            .get(inner.choice % inner.addresses.len())
            .unwrap_or(&API_ADDRESS)
    }

    pub fn has_tried_current_address(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.tried_current
    }

    pub async fn select_new_address(&self) {
        {
            let mut transaction = AddressCacheTransaction::new(self.inner.clone());

            transaction.choice = transaction.previous_cache.choice.wrapping_add(1);
            if transaction.choice == transaction.previous_cache.choice {
                return;
            }
            transaction.tried_current = false;

            tokio::task::block_in_place(move || {
                if (*self.change_listener)(Self::get_address_inner(&transaction)).is_err() {
                    log::error!("Failed to select a new API endpoint");
                    return;
                }
                transaction.commit();
            });
        }

        if let Err(error) = self.save_to_disk().await {
            log::error!("{}", error.display_chain());
        }
    }

    /// Forgets the currently selected address and randomizes
    /// the entire list.
    pub async fn randomize(&self) -> Result<(), Error> {
        {
            let mut transaction = AddressCacheTransaction::new(self.inner.clone());
            transaction.shuffle();
            transaction.choice = 0;

            let current_address = Self::get_address_inner(&transaction.previous_cache);
            let new_address = Self::get_address_inner(&transaction);

            tokio::task::block_in_place(move || {
                if new_address != current_address {
                    transaction.tried_current = false;
                    if (*self.change_listener)(new_address).is_err() {
                        return Err(Error::ChangeListenerError);
                    }
                }

                transaction.commit();
                Ok(())
            })?;
        }
        self.save_to_disk().await.map_err(Error::WriteAddressCache)
    }

    pub async fn set_addresses(&self, mut addresses: Vec<SocketAddr>) -> io::Result<()> {
        let should_update = {
            let mut transaction = AddressCacheTransaction::new(self.inner.clone());

            addresses.sort();

            let mut current_sorted = transaction.addresses.clone();
            current_sorted.sort();

            if addresses != current_sorted {
                let current_address = Self::get_address_inner(&transaction);

                transaction.addresses = addresses.clone();
                transaction.shuffle();

                // Prefer a likely-working address
                let choice = transaction
                    .addresses
                    .iter()
                    .position(|&addr| addr == current_address);
                if let Some(choice) = choice {
                    transaction.choice = choice;
                    transaction.commit();
                } else {
                    transaction.choice = 0;
                    transaction.tried_current = false;

                    tokio::task::block_in_place(move || {
                        if (*self.change_listener)(Self::get_address_inner(&transaction)).is_err() {
                            log::error!("Failed to select a new API endpoint");
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "callback returned an error",
                            ));
                        }
                        transaction.commit();
                        Ok(())
                    })?;
                }

                true
            } else {
                false
            }
        };
        if should_update {
            log::trace!("API address cache: {:?}", addresses);
            self.save_to_disk().await?;
        }
        Ok(())
    }

    async fn save_to_disk(&self) -> io::Result<()> {
        let write_path = match self.write_path.as_ref() {
            Some(write_path) => write_path,
            None => return Ok(()),
        };

        let (mut addresses, choice) = {
            let inner = self.inner.lock().unwrap();
            (inner.addresses.clone(), inner.choice)
        };

        // Place the current choice on top
        if !addresses.is_empty() {
            let addresses_len = addresses.len();
            addresses.swap(0, choice % addresses_len);
        }

        let temp_path = write_path.with_file_name("api-cache.temp");

        let mut file = fs::File::create(&temp_path).await?;
        let mut contents = addresses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n");
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
    addresses: Vec<SocketAddr>,
    choice: usize,
    tried_current: bool,
}

impl AddressCacheInner {
    fn from_addresses(addresses: Vec<SocketAddr>) -> Result<Self, Error> {
        if addresses.is_empty() {
            return Err(Error::EmptyAddressCache);
        }
        Ok(Self {
            addresses,
            choice: 0,
            tried_current: false,
        })
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        (&mut self.addresses[..]).shuffle(&mut rng);
    }

    /// Shuffle all but the first element
    fn shuffle_tail(&mut self) {
        let mut rng = rand::thread_rng();
        (&mut self.addresses[1..]).shuffle(&mut rng);
    }
}

struct AddressCacheTransaction {
    cache: Arc<Mutex<AddressCacheInner>>,
    working_cache: AddressCacheInner,
    previous_cache: AddressCacheInner,
}

impl AddressCacheTransaction {
    fn new(cache: Arc<Mutex<AddressCacheInner>>) -> Self {
        let current = { cache.lock().unwrap().clone() };
        Self {
            working_cache: current.clone(),
            previous_cache: current,
            cache,
        }
    }

    fn commit(self) {
        *self.cache.lock().unwrap() = self.working_cache;
    }
}

impl Deref for AddressCacheTransaction {
    type Target = AddressCacheInner;

    fn deref(&self) -> &Self::Target {
        &self.working_cache
    }
}

impl DerefMut for AddressCacheTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.working_cache
    }
}

async fn read_address_file(path: &Path) -> Result<Vec<SocketAddr>, Error> {
    let file = fs::File::open(path)
        .await
        .map_err(|error| Error::OpenAddressCache(error))?;
    let mut lines = BufReader::new(file).lines();
    let mut addresses = vec![];
    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|error| Error::ReadAddressCache(error))?
    {
        match line.trim().parse() {
            Ok(address) => addresses.push(address),
            Err(err) => {
                log::error!("Failed to parse cached address line: {}", err);
            }
        }
    }
    Ok(addresses)
}
