//! TOML configuration file

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{fs, io};

/// Path to the configuration file. Currently a file in the working directory.
const CONFIG_FILENAME: &str = "config.toml";

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    /// URLs to use as bases for installers.
    /// Files are expected at (example): `<base>/MullvadVPN-2025.1.exe`.
    pub base_urls: Vec<String>,
}

impl Config {
    pub async fn load_or_create() -> anyhow::Result<Self> {
        println!("Reading {CONFIG_FILENAME}");

        match fs::read_to_string(CONFIG_FILENAME).await {
            Ok(toml_str) => toml::from_str(&toml_str).context("Failed to parse TOML file"),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                println!("Creating default {CONFIG_FILENAME}");
                Ok(Self::default())
            }
            Err(err) => Err(err).context(format!("Failed to read {CONFIG_FILENAME}")),
        }
    }
}
