use super::API_ADDRESS;
use rand::seq::SliceRandom;
use std::{
    io,
    net::SocketAddr,
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
}

#[derive(Clone)]
pub struct AddressCache {
    inner: Arc<Mutex<AddressCacheInner>>,
    write_path: Option<Arc<Path>>,
}

impl AddressCache {
    /// Initialize cache using the given list, and write changes to `write_path`.
    pub fn new(addresses: Vec<SocketAddr>, write_path: Option<Box<Path>>) -> Result<Self, Error> {
        let mut cache = AddressCacheInner::from_addresses(addresses)?;
        cache.shuffle_tail();
        log::trace!("API address cache: {:?}", cache.addresses);
        log::debug!("Using API address: {:?}", Self::get_address_inner(&cache));

        let address_cache = Self {
            inner: Arc::new(Mutex::new(cache)),
            write_path: write_path.map(|cache| Arc::from(cache)),
        };
        Ok(address_cache)
    }

    /// Initialize cache using `read_path`, and write changes to `write_path`.
    pub async fn from_file(read_path: &Path, write_path: Option<Box<Path>>) -> Result<Self, Error> {
        log::debug!("Loading API addresses from {:?}", read_path);
        Self::new(read_address_file(read_path).await?, write_path)
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
            return API_ADDRESS.into();
        }
        *inner
            .addresses
            .get(inner.choice % inner.addresses.len())
            .unwrap_or(&API_ADDRESS.into())
    }

    pub fn has_tried_current_address(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.tried_current
    }

    pub async fn select_new_address(&self) {
        let (new_choice, old_choice) = {
            let mut inner = self.inner.lock().unwrap();
            let old_choice = inner.choice;
            inner.choice = inner.choice.wrapping_add(1);
            (inner.choice, old_choice)
        };

        if new_choice == old_choice {
            return;
        }

        if let Err(error) = self.save_to_disk().await {
            log::error!("{}", error.display_chain());
        }
    }

    /// Forgets the currently selected address and randomizes
    /// the entire list.
    pub async fn randomize(&self) -> Result<(), Error> {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.shuffle();
            inner.choice = 0;
            inner.tried_current = false;
        }
        self.save_to_disk().await.map_err(Error::WriteAddressCache)
    }

    pub async fn set_addresses(&self, mut addresses: Vec<SocketAddr>) -> io::Result<()> {
        let should_update = {
            let mut inner = self.inner.lock().unwrap();
            addresses.sort();
            let mut current_sorted = inner.addresses.clone();
            current_sorted.sort();
            if addresses != current_sorted {
                let current_address = Self::get_address_inner(&inner);

                inner.addresses = addresses.clone();
                inner.shuffle();

                // Prefer a likely-working address
                let choice = inner.addresses.iter().position(|&addr| addr == current_address);
                if let Some(choice) = choice {
                    inner.choice = choice;
                } else {
                    inner.choice = 0;
                    inner.tried_current = false;
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
