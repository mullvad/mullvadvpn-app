use std::{
    io,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

const FALLBACK_API_ADDRESS: (IpAddr, u16) = (crate::API_IP, 443);

#[derive(Clone)]
pub struct AddressCache {
    inner: Arc<Mutex<AddressCacheInner>>,
    cache_path: Option<Arc<Path>>,
}

impl AddressCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Default::default())),
            cache_path: None,
        }
    }

    pub async fn with_cache(cache_path: Box<Path>) -> Self {
        let cache = AddressCacheInner::from_cache_file(&cache_path)
            .await
            .unwrap_or_default();
        let inner = Arc::new(Mutex::new(cache));
        let cache_path = Some(cache_path.into());
        Self { inner, cache_path }
    }

    pub fn get_address(&self) -> SocketAddr {
        let mut inner = self.inner.lock().unwrap();
        inner.last_try = Some(inner.choice);

        Self::get_address_inner(&inner)
    }

    fn get_address_inner(inner: &AddressCacheInner) -> SocketAddr {
        if inner.addresses.is_empty() {
            return FALLBACK_API_ADDRESS.into();
        }
        *inner
            .addresses
            .get(inner.choice % inner.addresses.len())
            .unwrap_or(&FALLBACK_API_ADDRESS.into())
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

    pub async fn set_addresses(&self, addresses: Vec<SocketAddr>) -> io::Result<()> {
        let should_update = {
            let mut inner = self.inner.lock().unwrap();
            if addresses != inner.addresses {
                inner.addresses = addresses.clone();
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
    async fn from_cache_file(path: &Path) -> io::Result<Self> {
        let file = fs::File::open(path).await?;
        let mut lines = BufReader::new(file).lines();
        let mut addresses = vec![];
        while let Some(line) = lines.next_line().await? {
            // for line in lines.next_line() {
            match line.trim().parse() {
                Ok(address) => addresses.push(address),
                Err(err) => {
                    log::error!("Failed to parse cached address line: {}", err);
                }
            }
        }

        Ok(Self {
            addresses,
            ..Default::default()
        })
    }
}

impl Default for AddressCacheInner {
    fn default() -> Self {
        Self {
            addresses: vec![FALLBACK_API_ADDRESS.into()],
            choice: 0,
            last_try: None,
        }
    }
}
