//! TOML configuration file

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{fs, io};

/// Path to the configuration file. Currently a file in the working directory.
const CONFIG_FILENAME: &str = "meta.toml";

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    /// URLs to use as bases for installers.
    /// Files are expected at (example): `<base>/<version>/MullvadVPN-<version>.exe`.
    pub base_urls: Vec<String>,
}

impl Config {
    /// Try to load [CONFIG_FILENAME] from the working directory, create one if it does not exist.
    pub async fn load_or_create() -> anyhow::Result<Self> {
        match fs::read_to_string(CONFIG_FILENAME).await {
            Ok(toml_str) => toml::from_str(&toml_str).context("Failed to parse TOML file"),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                eprintln!("Creating default {CONFIG_FILENAME}");
                let self_ = Self::default();
                self_.save().await?;
                Ok(self_)
            }
            Err(err) => Err(err).context(format!("Failed to read {CONFIG_FILENAME}")),
        }
    }

    async fn save(&self) -> anyhow::Result<()> {
        let toml_str = toml::to_string_pretty(self).expect("Expected valid toml");
        fs::write(CONFIG_FILENAME, toml_str.as_bytes())
            .await
            .context(format!("Failed to save {CONFIG_FILENAME}"))
    }
}
