use mullvad_types::account::AccountToken;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

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
pub struct AccountHistory {
    accounts: Vec<AccountToken>,
    #[serde(skip)]
    cache_path: PathBuf,
}

impl AccountHistory {
    /// Returns a new empty `AccountHistory` ready to load from, or save to, the given cache dir.
    pub fn new(cache_dir: &Path) -> AccountHistory {
        AccountHistory {
            accounts: Vec::new(),
            cache_path: cache_dir.join(ACCOUNT_HISTORY_FILE),
        }
    }

    /// Loads account history from file. If no file is present this does nothing.
    pub fn load(&mut self) -> Result<()> {
        match File::open(&self.cache_path).map(io::BufReader::new) {
            Ok(mut file) => {
                log::info!(
                    "Loading account history from {}",
                    &self.cache_path.display()
                );
                self.accounts = Self::parse(&mut file)?.accounts;
                Ok(())
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                log::info!("No account history file at {}", &self.cache_path.display());
                Ok(())
            }
            Err(e) => Err(e).chain_err(|| ErrorKind::ReadError(self.cache_path.clone())),
        }
    }

    fn parse(file: &mut impl io::Read) -> Result<AccountHistory> {
        serde_json::from_reader(file).chain_err(|| ErrorKind::ParseError)
    }

    pub fn get_accounts(&self) -> &[AccountToken] {
        &self.accounts
    }

    /// Add account token to the account history removing duplicate entries
    pub fn add_account_token(&mut self, account_token: AccountToken) -> Result<()> {
        self.accounts
            .retain(|existing_token| existing_token != &account_token);
        self.accounts.push(account_token);

        let num_accounts = self.accounts.len();
        if num_accounts > ACCOUNT_HISTORY_LIMIT {
            self.accounts = self
                .accounts
                .split_off(num_accounts - ACCOUNT_HISTORY_LIMIT);
        }

        self.save()
    }

    /// Remove account token from the account history
    pub fn remove_account_token(&mut self, account_token: &AccountToken) -> Result<()> {
        self.accounts
            .retain(|existing_token| existing_token != account_token);
        self.save()
    }

    /// Serializes the account history and saves it to the file it was loaded from.
    fn save(&self) -> Result<()> {
        log::debug!("Writing account history to {}", self.cache_path.display());
        let mut file = File::create(&self.cache_path)
            .map(io::BufWriter::new)
            .chain_err(|| ErrorKind::WriteError(self.cache_path.clone()))?;

        serde_json::to_writer_pretty(&mut file, self)
            .chain_err(|| ErrorKind::WriteError(self.cache_path.clone()))?;

        file.get_mut()
            .sync_all()
            .chain_err(|| ErrorKind::WriteError(self.cache_path.clone()))
    }
}
