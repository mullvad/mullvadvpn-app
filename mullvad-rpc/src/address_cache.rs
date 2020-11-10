use super::API_ADDRESS;
use rand::seq::SliceRandom;
use std::{
    io,
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex},
};
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

    #[error(display = "The address cache is empty")]
    EmptyAddressCache,
}

#[derive(Clone)]
pub struct AddressCache {
    inner: Arc<Mutex<AddressCacheInner>>,
    cache_path: Option<Arc<Path>>,
}

impl AddressCache {
    /// Initialize cache using the given list, and write changes to `cache_path`.
    pub fn new(addresses: Vec<SocketAddr>, cache_path: Option<Box<Path>>) -> Result<Self, Error> {
        log::trace!("Using API addresses: {:?}", addresses);
        let cache = AddressCacheInner::from_addresses(addresses)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(cache)),
            cache_path: cache_path.map(|cache| Arc::from(cache)),
        })
    }

    /// Initialize cache using `read_path`, and write changes to `cache_path`.
    pub async fn from_file(read_path: &Path, cache_path: Option<Box<Path>>) -> Result<Self, Error> {
        log::trace!("Loading API addresses from {:?}", read_path);
        Self::new(read_address_file(read_path).await?, cache_path)
    }

    pub fn get_address(&self) -> SocketAddr {
        let mut inner = self.inner.lock().unwrap();
        inner.last_try = Some(inner.choice);

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

    pub fn register_failure(&self, failed_addr: SocketAddr, err: &dyn std::error::Error) {
        let mut inner = self.inner.lock().unwrap();

        let current_address = Self::get_address_inner(&inner);
        // Only choose the next server if the current one has been tried before and it failed
        if failed_addr == current_address
            && inner
                .last_try
                .map(|last_try| last_try == inner.choice)
                .unwrap_or(false)
        {
            log::error!("HTTP request failed: {}, will try next API address", err);
            inner.choice = inner.choice.wrapping_add(1);
        }
    }

    pub async fn set_addresses(&self, mut addresses: Vec<SocketAddr>) -> io::Result<()> {
        let should_update = {
            let mut inner = self.inner.lock().unwrap();
            addresses.sort();
            let mut current_sorted = inner.addresses.clone();
            current_sorted.sort();
            if addresses != current_sorted {
                inner.addresses = addresses.clone();
                inner.shuffle();
                inner.choice = 0;
                true
            } else {
                false
            }
        };
        if should_update {
            self.save_to_disk(addresses).await?;
        }
        Ok(())
    }

    async fn save_to_disk(&self, addresses: Vec<SocketAddr>) -> io::Result<()> {
        if let Some(cache_path) = self.cache_path.as_ref() {
            let mut file = fs::File::create(cache_path).await?;
            let mut contents = addresses
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n");
            contents += "\n";

            file.write_all(contents.as_bytes()).await?;
            file.sync_data().await?;
        }

        Ok(())
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
    last_try: Option<usize>,
}

impl AddressCacheInner {
    fn from_addresses(addresses: Vec<SocketAddr>) -> Result<Self, Error> {
        if addresses.is_empty() {
            return Err(Error::EmptyAddressCache);
        }
        let mut cache = Self {
            addresses,
            choice: 0,
            last_try: None,
        };
        cache.shuffle();
        Ok(cache)
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        (&mut self.addresses[..]).shuffle(&mut rng);
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
        // for line in lines.next_line() {
        match line.trim().parse() {
            Ok(address) => addresses.push(address),
            Err(err) => {
                log::error!("Failed to parse cached address line: {}", err);
            }
        }
    }
    Ok(addresses)
}
