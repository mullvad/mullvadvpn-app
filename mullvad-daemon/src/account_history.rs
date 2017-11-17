extern crate serde_json;

use app_dirs::{self, AppDataType};
use std::fs::File;
use std::io;
use std::path::PathBuf;

use mullvad_types::account::AccountToken;

error_chain! {
    errors {
        DirectoryError {
            description("Unable to create account history directory for program")
        }
        ReadError(path: PathBuf) {
            description("Unable to read account history file")
            display("Unable to read account history from {}", path.to_string_lossy())
        }
        WriteError(path: PathBuf) {
            description("Unable to write account history file")
            display("Unable to write account history to {}", path.to_string_lossy())
        }
        ParseError {
            description("Malformed account history")
        }
    }
}

static ACCOUNT_HISTORY_FILE: &str = "account-history.json";
static ACCOUNT_HISTORY_LIMIT: usize = 3;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AccountHistory {
    accounts: Vec<AccountToken>,
}

impl AccountHistory {
    /// Loads account history from file. If no file is present it returns the defaults.
    pub fn load() -> Result<AccountHistory> {
        let history_path = Self::get_path()?;
        match File::open(&history_path) {
            Ok(mut file) => {
                info!(
                    "Loading account history from {}",
                    history_path.to_string_lossy()
                );
                Self::parse(&mut file)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!(
                    "No account history file at {}, using defaults",
                    history_path.to_string_lossy()
                );
                Ok(AccountHistory::default())
            }
            Err(e) => Err(e).chain_err(|| ErrorKind::ReadError(history_path)),
        }
    }

    pub fn get_accounts(&self) -> Vec<AccountToken> {
        self.accounts.clone()
    }

    /// Add account token to the account history removing duplicate entries
    pub fn add_account_token(&mut self, account_token: AccountToken) -> Result<()> {
        self.accounts
            .retain(|existing_token| existing_token != &account_token);
        self.accounts.push(account_token);

        let num_accounts = self.accounts.len();
        if num_accounts > ACCOUNT_HISTORY_LIMIT {
            self.accounts = self.accounts
                .split_off(num_accounts - ACCOUNT_HISTORY_LIMIT);
        }

        self.save()
    }

    /// Remove account token from the account history
    pub fn remove_account_token(&mut self, account_token: AccountToken) -> Result<()> {
        self.accounts
            .retain(|existing_token| existing_token != &account_token);
        self.save()
    }

    /// Serializes the account history and saves it to the file it was loaded from.
    fn save(&self) -> Result<()> {
        let path = Self::get_path()?;

        debug!("Writing account history to {}", path.to_string_lossy());
        let file = File::create(&path).chain_err(|| ErrorKind::WriteError(path.clone()))?;

        serde_json::to_writer_pretty(file, self).chain_err(|| ErrorKind::WriteError(path))
    }

    fn parse(file: &mut File) -> Result<AccountHistory> {
        serde_json::from_reader(file).chain_err(|| ErrorKind::ParseError)
    }

    fn get_path() -> Result<PathBuf> {
        let dir = app_dirs::app_root(AppDataType::UserCache, &::APP_INFO)
            .chain_err(|| ErrorKind::DirectoryError)?;
        Ok(dir.join(ACCOUNT_HISTORY_FILE))
    }
}
