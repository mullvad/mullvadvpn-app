#[cfg(target_os = "android")]
use futures::future::{Executor, Future};
#[cfg(not(target_os = "android"))]
use futures::{
    future::{self, Executor, Future},
    sync::oneshot,
};
use mullvad_rpc::{rest::MullvadRestHandle, WireguardKeyProxy};
use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use std::{
    collections::VecDeque,
    fs,
    io::{self, Seek, Write},
    path::Path,
};
use talpid_types::ErrorExt;
use tokio_core::reactor::Remote;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Unable to read account history file")]
    Read(#[error(source)] io::Error),

    #[error(display = "Failed to serialize account history")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Unable to write account history file")]
    Write(#[error(source)] io::Error),
}

static ACCOUNT_HISTORY_FILE: &str = "account-history.json";
static ACCOUNT_HISTORY_LIMIT: usize = 3;

/// A trivial MRU cache of account data
pub struct AccountHistory {
    file: io::BufWriter<fs::File>,
    accounts: VecDeque<AccountEntry>,
    rpc_handle: MullvadRestHandle,
    tokio_remote: Remote,
}


impl AccountHistory {
    pub fn new(
        cache_dir: &Path,
        rpc_handle: MullvadRestHandle,
        tokio_remote: Remote,
    ) -> Result<AccountHistory> {
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
        let path = cache_dir.join(ACCOUNT_HISTORY_FILE);
        log::info!("Opening account history file in {}", path.display());
        let mut reader = options
            .write(true)
            .read(true)
            .create(true)
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
        let file = io::BufWriter::new(reader.into_inner());
        let mut history = AccountHistory {
            file,
            accounts,
            rpc_handle,
            tokio_remote,
        };
        if let Err(e) = history.save_to_disk() {
            log::error!("Failed to save account cache after opening it: {}", e);
        }
        Ok(history)
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
    pub fn get(&mut self, account: &AccountToken) -> Result<Option<AccountEntry>> {
        let (idx, entry) = match self
            .accounts
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
        self.insert(entry.clone())?;
        Ok(Some(entry))
    }

    /// Bumps history of an account token. If the account token is not in history, it will be
    /// added.
    pub fn bump_history(&mut self, account: &AccountToken) -> Result<()> {
        if self.get(account)?.is_none() {
            let new_entry = AccountEntry {
                account: account.to_string(),
                wireguard: None,
            };
            self.insert(new_entry)?;
        }
        Ok(())
    }

    fn create_remove_wg_key_rpc(
        &self,
        account: &str,
        wg_data: &WireguardData,
    ) -> impl Future<Item = (), Error = ()> {
        let mut rpc = WireguardKeyProxy::new(self.rpc_handle.clone());
        rpc.remove_wireguard_key(String::from(account), &wg_data.private_key.public_key())
            .map_err(|e| log::error!("Failed to remove WireGuard key: {}", e))
    }

    /// Always inserts a new entry at the start of the list
    pub fn insert(&mut self, new_entry: AccountEntry) -> Result<()> {
        self.accounts
            .retain(|entry| entry.account != new_entry.account);

        self.accounts.push_front(new_entry);

        if self.accounts.len() > ACCOUNT_HISTORY_LIMIT {
            let last_entry = self.accounts.pop_back().unwrap();
            if let Some(wg_data) = last_entry.wireguard {
                let fut = self.create_remove_wg_key_rpc(&last_entry.account, &wg_data);
                if let Err(e) = self.tokio_remote.execute(fut) {
                    log::error!("Failed to spawn future to remove WireGuard key: {:?}", e);
                }
            }
        }

        self.save_to_disk()
    }

    /// Retrieve account history.
    pub fn get_account_history(&self) -> Vec<AccountToken> {
        self.accounts
            .iter()
            .map(|entry| entry.account.clone())
            .collect()
    }

    /// Remove account data
    pub fn remove_account(&mut self, account: &str) -> Result<()> {
        let entry = self.get(&String::from(account))?;
        let entry = match entry {
            Some(entry) => entry,
            None => return Ok(()),
        };

        if let Some(wg_data) = entry.wireguard {
            let fut = self.create_remove_wg_key_rpc(account, &wg_data);
            if let Err(e) = self.tokio_remote.execute(fut) {
                log::error!("Failed to spawn future to remove WireGuard key: {:?}", e);
            }
        }

        let _ = self.accounts.pop_front();
        self.save_to_disk()
    }

    /// Remove account history
    #[cfg(not(target_os = "android"))]
    pub fn clear(&mut self) -> Result<()> {
        let mut rpc = WireguardKeyProxy::new(self.rpc_handle.clone());

        log::debug!("account_history::clear");

        let mut removal_futures = Vec::with_capacity(ACCOUNT_HISTORY_LIMIT);

        for entry in self.accounts.iter() {
            if let Some(wg_data) = &entry.wireguard {
                let fut = rpc
                    .remove_wireguard_key(entry.account.clone(), &wg_data.private_key.public_key())
                    .map_err(|e| log::error!("Failed to remove WireGuard key: {}", e));
                removal_futures.push(fut);
            }
        }

        let joined_futs = future::join_all(removal_futures);
        let (tx, rx) = oneshot::channel();

        let execute_result = self.tokio_remote.execute(joined_futs.then(|result| {
            let _ = tx.send(result);
            Ok(())
        }));
        if let Err(e) = execute_result {
            log::error!("Failed to spawn future to remove WireGuard keys: {:?}", e);
        } else {
            let _ = rx.wait();
        }

        self.accounts = VecDeque::new();
        self.save_to_disk()
    }

    fn save_to_disk(&mut self) -> Result<()> {
        self.file.get_mut().set_len(0).map_err(Error::Write)?;
        self.file
            .seek(io::SeekFrom::Start(0))
            .map_err(Error::Write)?;
        serde_json::to_writer_pretty(&mut self.file, &self.accounts).map_err(Error::Serialize)?;
        self.file.flush().map_err(Error::Write)?;
        self.file.get_mut().sync_all().map_err(Error::Write)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountEntry {
    pub account: AccountToken,
    pub wireguard: Option<WireguardData>,
}
