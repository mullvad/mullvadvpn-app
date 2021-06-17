use crate::settings::SettingsPersister;
use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use regex::Regex;
use std::{
    fs,
    io::{self, Read, Seek, Write},
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

pub struct AccountHistory {
    file: Arc<Mutex<io::BufWriter<fs::File>>>,
    token: Option<AccountToken>,
}

lazy_static::lazy_static! {
    static ref ACCOUNT_REGEX: Regex = Regex::new(r"^[0-9]+$").unwrap();
}


impl AccountHistory {
    pub async fn new(
        cache_dir: &Path,
        settings_dir: &Path,
        settings: &mut SettingsPersister,
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
        let (file, token) = if path.is_file() {
            log::info!("Opening account history file in {}", path.display());
            let mut reader = options
                .write(true)
                .read(true)
                .open(path)
                .map(io::BufReader::new)
                .map_err(Error::Read)?;

            let mut buffer = String::new();
            let token: Option<AccountToken> = match reader.read_to_string(&mut buffer) {
                Ok(0) => None,
                Ok(_) if ACCOUNT_REGEX.is_match(&buffer) => Some(buffer),
                Ok(_) | Err(_) => {
                    log::warn!("Failed to parse account history. Trying old formats",);
                    match Self::try_format_v2(&mut reader)? {
                        Some((token, migrated_data)) => {
                            if let Err(error) = settings.set_wireguard(migrated_data).await {
                                log::error!(
                                    "{}",
                                    error.display_chain_with_msg(
                                        "Failed to migrate WireGuard key from account history"
                                    )
                                );
                            }
                            Some(token)
                        }
                        None => Self::try_format_v1(&mut reader)?,
                    }
                }
            };

            (reader.into_inner(), token)
        } else {
            log::info!("Creating account history file in {}", path.display());
            (
                options
                    .write(true)
                    .create(true)
                    .open(path)
                    .map_err(Error::Read)?,
                None,
            )
        };
        let file = io::BufWriter::new(file);
        let mut history = AccountHistory {
            file: Arc::new(Mutex::new(file)),
            token,
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

    fn try_format_v1(reader: &mut io::BufReader<fs::File>) -> Result<Option<AccountToken>> {
        #[derive(Deserialize)]
        struct OldFormat {
            accounts: Vec<AccountToken>,
        }
        reader.seek(io::SeekFrom::Start(0)).map_err(Error::Read)?;
        Ok(serde_json::from_reader(reader)
            .map(|old_format: OldFormat| old_format.accounts.first().cloned())
            .unwrap_or_else(|_| None))
    }

    fn try_format_v2(
        reader: &mut io::BufReader<fs::File>,
    ) -> Result<Option<(AccountToken, Option<WireguardData>)>> {
        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct AccountEntry {
            pub account: AccountToken,
            pub wireguard: Option<WireguardData>,
        }
        reader.seek(io::SeekFrom::Start(0)).map_err(Error::Read)?;
        Ok(serde_json::from_reader(reader)
            .map(|entries: Vec<AccountEntry>| {
                entries
                    .first()
                    .map(|entry| (entry.account.clone(), entry.wireguard.clone()))
            })
            .unwrap_or_else(|_| None))
    }

    /// Gets the account token in the history
    pub fn get(&self) -> Option<AccountToken> {
        self.token.clone()
    }

    /// Replace the account token in the history
    pub async fn set(&mut self, new_entry: AccountToken) -> Result<()> {
        self.token = Some(new_entry);
        self.save_to_disk().await
    }

    /// Remove account history
    pub async fn clear(&mut self) -> Result<()> {
        self.token = None;
        self.save_to_disk().await
    }

    async fn save_to_disk(&mut self) -> Result<()> {
        let file = self.file.clone();
        let token = self.token.clone();

        tokio::task::spawn_blocking(move || {
            let mut file = file.lock().unwrap();
            file.get_mut().set_len(0).map_err(Error::Write)?;
            file.seek(io::SeekFrom::Start(0)).map_err(Error::Write)?;
            if let Some(token) = token {
                write!(&mut file, "{}", token).map_err(Error::Write)?;
            }
            file.flush().map_err(Error::Write)?;
            file.get_mut().sync_all().map_err(Error::Write)
        })
        .await
        .map_err(Error::WriteCancelled)?
    }
}
