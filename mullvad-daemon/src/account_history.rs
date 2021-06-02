use mullvad_rpc::{rest::MullvadRestHandle, WireguardKeyProxy};
use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use std::{
    collections::VecDeque,
    fs,
    future::Future,
    io::{self, Seek, Write},
    path::Path,
    sync::{Arc, Mutex},
};
use talpid_types::ErrorExt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Unable to open or read account history file")]
    Read(#[error(source)] io::Error),

    #[error(display = "Failed to serialize account history")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Unable to write account history file")]
    Write(#[error(source)] io::Error),

    #[error(display = "Write task panicked or was cancelled")]
    WriteCancelled(#[error(source)] tokio::task::JoinError),
}

static ACCOUNT_HISTORY_FILE: &str = "account-history.json";
static ACCOUNT_HISTORY_LIMIT: usize = 1;

/// A trivial MRU cache of account data
pub struct AccountHistory {
    file: Arc<Mutex<io::BufWriter<fs::File>>>,
    accounts: Arc<Mutex<VecDeque<AccountEntry>>>,
    rpc_handle: MullvadRestHandle,
}


impl AccountHistory {
    pub async fn new(
        cache_dir: &Path,
        settings_dir: &Path,
        rpc_handle: MullvadRestHandle,
    ) -> Result<AccountHistory> {
        Self::migrate_from_old_file_location(cache_dir, settings_dir).await;

        let mut options = fs::OpenOptions::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // a share mode of zero ensures exclusive access to the file to *this* process
            options.share_mode(0);
        }
        let path = settings_dir.join(ACCOUNT_HISTORY_FILE);
        let (file, accounts) = if path.is_file() {
            log::info!("Opening account history file in {}", path.display());
            let mut reader = options
                .write(true)
                .read(true)
                .open(path)
                .map(io::BufReader::new)
                .map_err(Error::Read)?;

            let accounts: VecDeque<AccountEntry> = match serde_json::from_reader(&mut reader) {
                Err(e) => {
                    log::warn!(
                        "{}",
                        e.display_chain_with_msg("Failed to read+deserialize account history")
                    );
                    Self::try_old_format(&mut reader)?
                        .into_iter()
                        .map(|account| AccountEntry {
                            account,
                            wireguard: None,
                        })
                        .collect()
                }
                Ok(accounts) => accounts,
            };

            (reader.into_inner(), accounts)
        } else {
            log::info!("Creating account history file in {}", path.display());
            (
                options
                    .write(true)
                    .create(true)
                    .open(path)
                    .map_err(Error::Read)?,
                VecDeque::new(),
            )
        };
        let file = io::BufWriter::new(file);
        let mut history = AccountHistory {
            file: Arc::new(Mutex::new(file)),
            accounts: Arc::new(Mutex::new(accounts)),
            rpc_handle,
        };
        if let Err(e) = history.save_to_disk().await {
            log::error!("Failed to save account cache after opening it: {}", e);
        }
        Ok(history)
    }

    async fn migrate_from_old_file_location(old_dir: &Path, new_dir: &Path) {
        use tokio::fs;

        let old_path = old_dir.join(ACCOUNT_HISTORY_FILE);
        let new_path = new_dir.join(ACCOUNT_HISTORY_FILE);
        if !old_path.exists() || new_path.exists() || new_path == old_path {
            return;
        }

        if let Err(error) = fs::copy(&old_path, &new_path).await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to migrate account history file location")
            );
        } else {
            let _ = fs::remove_file(old_path).await;
        }
    }

    fn try_old_format(reader: &mut io::BufReader<fs::File>) -> Result<Vec<AccountToken>> {
        #[derive(Deserialize)]
        struct OldFormat {
            accounts: Vec<AccountToken>,
        }
        reader.seek(io::SeekFrom::Start(0)).map_err(Error::Read)?;
        Ok(serde_json::from_reader(reader)
            .map(|old_format: OldFormat| old_format.accounts)
            .unwrap_or_else(|_| Vec::new()))
    }

    /// Gets account data for a certain account id and bumps it's entry to the top of the list if
    /// it isn't there already. Returns None if the account entry is not available.
    pub async fn get(&mut self, account: &AccountToken) -> Result<Option<AccountEntry>> {
        let (idx, entry) = match self
            .accounts
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_idx, entry)| &entry.account == account)
        {
            Some((idx, entry)) => (idx, entry.clone()),
            None => {
                return Ok(None);
            }
        };
        // this account is already on top
        if idx == 0 {
            return Ok(Some(entry));
        }
        self.insert(entry.clone()).await?;
        Ok(Some(entry))
    }

    /// Bumps history of an account token. If the account token is not in history, it will be
    /// added.
    pub async fn bump_history(&mut self, account: &AccountToken) -> Result<()> {
        if self.get(account).await?.is_none() {
            let new_entry = AccountEntry {
                account: account.to_string(),
                wireguard: None,
            };
            self.insert(new_entry).await?;
        }
        Ok(())
    }

    fn create_remove_wg_key_rpc(
        &self,
        account: &str,
        wg_data: &WireguardData,
    ) -> impl Future<Output = ()> + 'static {
        let mut rpc = WireguardKeyProxy::new(self.rpc_handle.clone());
        let pub_key = wg_data.private_key.public_key();
        let account = String::from(account);

        async move {
            if let Err(err) = rpc.remove_wireguard_key(account, &pub_key).await {
                log::error!("Failed to remove WireGuard key: {}", err);
            }
        }
    }

    /// Always inserts a new entry at the start of the list
    pub async fn insert(&mut self, new_entry: AccountEntry) -> Result<()> {
        let mut accounts = self.accounts.lock().unwrap();
        accounts.retain(|entry| entry.account != new_entry.account);
        accounts.push_front(new_entry);

        while accounts.len() > ACCOUNT_HISTORY_LIMIT {
            let last_entry = accounts.pop_back().unwrap();
            if let Some(wg_data) = last_entry.wireguard {
                tokio::spawn(self.create_remove_wg_key_rpc(&last_entry.account, &wg_data));
            }
        }

        std::mem::drop(accounts);
        self.save_to_disk().await
    }

    /// Retrieve account history.
    pub fn get_account_history(&self) -> Vec<AccountToken> {
        self.accounts
            .lock()
            .unwrap()
            .iter()
            .map(|entry| entry.account.clone())
            .collect()
    }

    /// Remove account data
    pub async fn remove_account(&mut self, account: &str) -> Result<()> {
        let entry = self.get(&String::from(account)).await?;
        let entry = match entry {
            Some(entry) => entry,
            None => return Ok(()),
        };

        if let Some(wg_data) = entry.wireguard {
            tokio::spawn(self.create_remove_wg_key_rpc(account, &wg_data));
        }

        let _ = self.accounts.lock().unwrap().pop_front();
        self.save_to_disk().await
    }

    /// Remove account history
    pub async fn clear(&mut self) -> Result<()> {
        log::debug!("account_history::clear");

        let rpc = WireguardKeyProxy::new(self.rpc_handle.clone());

        let removal: Vec<_> = self
            .accounts
            .lock()
            .unwrap()
            .drain(0..)
            .filter_map(move |entry| {
                let account = entry.account.clone();
                let mut rpc = rpc.clone();
                entry.wireguard.map(move |wg_data| {
                    let public_key = wg_data.private_key.public_key();
                    async move {
                        if let Err(err) = rpc.remove_wireguard_key(account, &public_key).await {
                            log::error!("Failed to remove WireGuard key: {}", err);
                        }
                    }
                })
            })
            .collect();


        futures::future::join_all(removal).await;

        {
            let mut accounts = self.accounts.lock().unwrap();
            *accounts = VecDeque::new();
        }
        self.save_to_disk().await
    }

    async fn save_to_disk(&mut self) -> Result<()> {
        let file = self.file.clone();
        let accounts = self.accounts.clone();

        tokio::task::spawn_blocking(move || {
            let mut file = file.lock().unwrap();
            let accounts = accounts.lock().unwrap();
            Self::save_to_disk_inner(&mut *file, &*accounts)
        })
        .await
        .map_err(Error::WriteCancelled)?
    }

    fn save_to_disk_inner(
        mut file: &mut io::BufWriter<fs::File>,
        accounts: &VecDeque<AccountEntry>,
    ) -> Result<()> {
        file.get_mut().set_len(0).map_err(Error::Write)?;
        file.seek(io::SeekFrom::Start(0)).map_err(Error::Write)?;
        serde_json::to_writer_pretty(&mut file, accounts).map_err(Error::Serialize)?;
        file.flush().map_err(Error::Write)?;
        file.get_mut().sync_all().map_err(Error::Write)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountEntry {
    pub account: AccountToken,
    pub wireguard: Option<WireguardData>,
}
