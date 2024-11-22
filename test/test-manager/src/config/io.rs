//! See [ConfigFile].

use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use super::{Config, Error};

/// On-disk representation of [Config].
pub struct ConfigFile {
    path: PathBuf,
    config: Config,
}

impl ConfigFile {
    /// Make config changes and save them to disk
    pub async fn edit(&mut self, edit: impl FnOnce(&mut Config)) -> Result<(), Error> {
        Self::ensure_config_dir().await?;
        edit(&mut self.config);
        self.config_save().await
    }

    /// Make config changes and save them to disk
    pub async fn load_or_default() -> Result<Self, Error> {
        let path = Self::get_config_path()?;
        let config = Self::config_load_or_default(&path).await?;
        let config_file = Self { path, config };
        Ok(config_file)
    }

    async fn config_load_or_default<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        Self::config_load(path).await.or_else(|error| match error {
            Error::Read(ref io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                log::trace!("Failed to read config file");
                Ok(Config::default())
            }
            error => Err(error),
        })
    }

    async fn config_load<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        let data = tokio::fs::read(path).await.map_err(Error::Read)?;
        serde_json::from_slice(&data).map_err(Error::InvalidConfig)
    }

    async fn config_save(&self) -> Result<(), Error> {
        let data = serde_json::to_vec_pretty(&self.config).unwrap();
        tokio::fs::write(&self.path, &data)
            .await
            .map_err(Error::Write)
    }

    /// Get configuration file path
    fn get_config_path() -> Result<PathBuf, Error> {
        Ok(Self::get_config_dir()?.join("config.json"))
    }

    /// Get configuration file directory
    fn get_config_dir() -> Result<PathBuf, Error> {
        let dir = dirs::config_dir()
            .ok_or(Error::FindConfigDir)?
            .join("mullvad-test");
        Ok(dir)
    }

    /// Create configuration file directory if it does not exist
    async fn ensure_config_dir() -> Result<(), Error> {
        tokio::fs::create_dir_all(Self::get_config_dir()?)
            .await
            .map_err(Error::CreateConfigDir)
    }
}

impl Deref for ConfigFile {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
