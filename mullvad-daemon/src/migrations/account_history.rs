use super::{Error, Result};
use mullvad_types::{account::AccountToken, wireguard::WireguardData};
use regex::Regex;
use std::path::Path;
use talpid_types::ErrorExt;
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};


const ACCOUNT_HISTORY_FILE: &str = "account-history.json";

lazy_static::lazy_static! {
    static ref ACCOUNT_REGEX: Regex = Regex::new(r"^[0-9]+$").unwrap();
}


pub async fn migrate_location(old_dir: &Path, new_dir: &Path) {
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

pub async fn migrate_formats(settings_dir: &Path, settings: &mut serde_json::Value) -> Result<()> {
    let path = settings_dir.join(ACCOUNT_HISTORY_FILE);
    if !path.is_file() {
        return Ok(());
    }

    let mut options = fs::OpenOptions::new();
    #[cfg(unix)]
    {
        options.mode(0o600);
    }
    let mut file = options
        .write(true)
        .read(true)
        .open(path)
        .await
        .map_err(Error::ReadHistoryError)?;

    let mut bytes = vec![];
    file.read_to_end(&mut bytes)
        .await
        .map_err(Error::ReadHistoryError)?;

    if is_format_v3(&bytes) {
        return Ok(());
    }

    let token = if let Some((token, wg_data)) = try_format_v2(&bytes) {
        settings["wireguard"] = serde_json::json!(wg_data);
        token
    } else if let Some(token) = try_format_v1(&bytes) {
        token
    } else {
        return Err(Error::ParseHistoryError);
    };

    file.set_len(0).await.map_err(Error::WriteHistoryError)?;
    file.seek(io::SeekFrom::Start(0))
        .await
        .map_err(Error::WriteHistoryError)?;
    file.write_all(token.as_bytes())
        .await
        .map_err(Error::WriteHistoryError)?;
    file.flush().await.map_err(Error::WriteHistoryError)?;
    file.sync_all().await.map_err(Error::WriteHistoryError)?;

    Ok(())
}

fn is_format_v3(bytes: &[u8]) -> bool {
    match std::str::from_utf8(bytes) {
        Ok(token) => token.is_empty() || ACCOUNT_REGEX.is_match(token),
        Err(_) => false,
    }
}

fn try_format_v2(bytes: &[u8]) -> Option<(AccountToken, Option<WireguardData>)> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AccountEntry {
        pub account: AccountToken,
        pub wireguard: Option<WireguardData>,
    }
    serde_json::from_slice(bytes)
        .map(|entries: Vec<AccountEntry>| {
            entries
                .first()
                .map(|entry| (entry.account.clone(), entry.wireguard.clone()))
        })
        .unwrap_or(None)
}

fn try_format_v1(bytes: &[u8]) -> Option<AccountToken> {
    #[derive(Deserialize)]
    struct OldFormat {
        accounts: Vec<AccountToken>,
    }
    serde_json::from_slice(bytes)
        .map(|old_format: OldFormat| old_format.accounts.first().cloned())
        .unwrap_or(None)
}
