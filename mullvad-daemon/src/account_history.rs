use mullvad_types::account::AccountToken;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use talpid_types::ErrorExt;
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to open or read account history file")]
    Read(#[source] io::Error),

    #[error("Failed to serialize account history")]
    Serialize(#[source] serde_json::Error),

    #[error("Unable to write account history file")]
    Write(#[source] io::Error),

    #[error("Write task panicked or was cancelled")]
    WriteCancelled(#[source] tokio::task::JoinError),
}

static ACCOUNT_HISTORY_FILE: &str = "account-history.json";

pub struct AccountHistory {
    file: io::BufWriter<fs::File>,
    token: Option<AccountToken>,
}

static ACCOUNT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9]+$").unwrap());

impl AccountHistory {
    pub async fn new(
        settings_dir: &Path,
        current_token: Option<AccountToken>,
    ) -> Result<AccountHistory> {
        let mut options = fs::OpenOptions::new();
        #[cfg(unix)]
        {
            options.mode(0o600);
        }
        #[cfg(windows)]
        {
            // a share mode of zero ensures exclusive access to the file to *this* process
            options.share_mode(0);
        }

        let path = settings_dir.join(ACCOUNT_HISTORY_FILE);
        log::info!("Opening account history file in {}", path.display());
        let mut reader = options
            .write(true)
            .create(true)
            .read(true)
            .open(path)
            .await
            .map(io::BufReader::new)
            .map_err(Error::Read)?;

        let mut buffer = String::new();
        let (token, should_save): (Option<AccountToken>, bool) =
            match reader.read_to_string(&mut buffer).await {
                Ok(_) if ACCOUNT_REGEX.is_match(&buffer) => (Some(buffer), false),
                Ok(0) => (current_token, true),
                Ok(_) | Err(_) => {
                    log::warn!("Failed to parse account history");
                    (current_token, true)
                }
            };

        let file = io::BufWriter::new(reader.into_inner());
        let mut history = AccountHistory { file, token };
        if should_save {
            if let Err(error) = history.save_to_disk().await {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to save account history after opening it")
                );
            }
        }
        Ok(history)
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
        self.file.get_mut().set_len(0).await.map_err(Error::Write)?;
        self.file
            .seek(io::SeekFrom::Start(0))
            .await
            .map_err(Error::Write)?;
        if let Some(ref token) = self.token {
            self.file
                .write_all(token.as_bytes())
                .await
                .map_err(Error::Write)?;
        }
        self.file.flush().await.map_err(Error::Write)?;
        self.file.get_mut().sync_all().await.map_err(Error::Write)
    }
}
