extern crate serde_json;

use std::fs::File;
use std::io;
use std::path::{PathBuf, Path};

use mullvad_types::account::AccountToken;

error_chain! {
    errors {
        ReadError(path: PathBuf) {
            description("Unable to read account history file")
            display("Unable to read account history from {}", path.display())
        }
        WriteError(path: PathBuf) {
            description("Unable to write account history file")
            display("Unable to write account history to {}", path.display())
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
    pub fn load(cache_dir: &Path) -> Result<AccountHistory> {
        let history_path = cache_dir.join(ACCOUNT_HISTORY_FILE);
        match File::open(&history_path) {
            Ok(file) => {
                info!(
                    "Loading account history from {}",
                    history_path.display()
                );
                Self::parse(&mut io::BufReader::new(file))
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!(
                    "No account history file at {}, using defaults",
                    history_path.display()
                );
                Ok(AccountHistory::default())
            }
            Err(e) => Err(e).chain_err(|| ErrorKind::ReadError(history_path)),
        }
    }

    pub fn get_accounts(&self) -> &[AccountToken] {
        &self.accounts
    }

    /// Add account token to the account history removing duplicate entries
    pub fn add_account_token(&mut self, account_token: AccountToken, cache_dir: &Path) -> Result<()> {
        self.accounts
            .retain(|existing_token| existing_token != &account_token);
        self.accounts.push(account_token);

        let num_accounts = self.accounts.len();
        if num_accounts > ACCOUNT_HISTORY_LIMIT {
            self.accounts = self
                .accounts
                .split_off(num_accounts - ACCOUNT_HISTORY_LIMIT);
        }

        self.save(cache_dir)
    }

    /// Remove account token from the account history
    pub fn remove_account_token(&mut self, account_token: AccountToken, cache_dir: &Path) -> Result<()> {
        self.accounts
            .retain(|existing_token| existing_token != &account_token);
        self.save(cache_dir)
    }

    /// Serializes the account history and saves it to the file it was loaded from.
    fn save(&self, cache_dir: &Path) -> Result<()> {
        let path = cache_dir.join(ACCOUNT_HISTORY_FILE);

        debug!("Writing account history to {}", path.display());
        let file = File::create(&path).chain_err(|| ErrorKind::WriteError(path.clone()))?;

        serde_json::to_writer_pretty(io::BufWriter::new(file), self).chain_err(|| ErrorKind::WriteError(path))
    }

    fn parse(file: &mut impl io::Read) -> Result<AccountHistory> {
        serde_json::from_reader(file).chain_err(|| ErrorKind::ParseError)
    }
}
