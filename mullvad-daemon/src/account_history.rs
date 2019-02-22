use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use std::{
    collections::VecDeque,
    fs,
    io::{self, Seek, Write},
    path::Path,
};

error_chain! {
    errors {
        ReadError {
            description("Unable to read account history file")
        }
        WriteError {
            description("Unable to write account history file")
        }
    }
}

static ACCOUNT_HISTORY_FILE: &str = "account-history.json";
static ACCOUNT_HISTORY_LIMIT: usize = 3;

/// A trivial MRU cache of account data
pub struct AccountHistory {
    file: io::BufWriter<fs::File>,
    accounts: VecDeque<AccountEntry>,
}


impl AccountHistory {
    pub fn new(cache_dir: &Path) -> Result<AccountHistory> {
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
            .chain_err(|| ErrorKind::ReadError)?;

        let accounts: VecDeque<AccountEntry> = match serde_json::from_reader(&mut reader) {
            Err(e) => {
                log::error!("Failed to read account history - {}", e);
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
        Ok(AccountHistory { file, accounts })
    }

    fn try_old_format(reader: &mut io::BufReader<fs::File>) -> Result<Vec<AccountToken>> {
        reader
            .seek(io::SeekFrom::Start(0))
            .chain_err(|| ErrorKind::ReadError)?;
        Ok(serde_json::from_reader(reader).unwrap_or(vec![]))
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
        if let None = self.get(account)? {
            let new_entry = AccountEntry {
                account: account.to_string(),
                wireguard: None,
            };
            self.insert(new_entry)?;
        }
        Ok(())
    }

    /// Always inserts a new entry at the start of the list
    pub fn insert(&mut self, new_entry: AccountEntry) -> Result<()> {
        self.accounts
            .retain(|entry| entry.account != new_entry.account);

        self.accounts.push_front(new_entry);
        if self.accounts.len() > ACCOUNT_HISTORY_LIMIT {
            let _ = self.accounts.pop_back();
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
        self.accounts.retain(|entry| entry.account != account);
        self.save_to_disk()
    }

    fn save_to_disk(&mut self) -> Result<()> {
        self.file
            .get_mut()
            .set_len(0)
            .chain_err(|| ErrorKind::WriteError)?;
        self.file
            .seek(io::SeekFrom::Start(0))
            .chain_err(|| ErrorKind::WriteError)?;
        serde_json::to_writer_pretty(&mut self.file, &self.accounts)
            .chain_err(|| ErrorKind::WriteError)?;
        self.file.flush().chain_err(|| ErrorKind::WriteError)?;
        self.file
            .get_mut()
            .sync_all()
            .chain_err(|| ErrorKind::WriteError)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountEntry {
    pub account: AccountToken,
    pub wireguard: Option<WireguardData>,
}
