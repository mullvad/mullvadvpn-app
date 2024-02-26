use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Error {
    #[error("Failed open file for writing")]
    OpenFile,

    #[error("Failed to write downloaded file to disk")]
    WriteFile,

    #[error("Failed to convert download to bytes")]
    ToBytes,

    #[error("Failed to convert download to bytes")]
    RequestFailed,

    #[error("Cannot parse version")]
    InvalidVersion,

    #[error("Failed to run package installer")]
    RunApp,

    #[error("Failed to create temporary uninstaller")]
    CreateTempUninstaller,

    #[error("Installer or uninstaller failed due to an unknown error: {0}")]
    InstallerFailed(i32),

    #[error("Installer or uninstaller failed due to a signal")]
    InstallerFailedSignal,

    #[error("Unrecognized OS: {0}")]
    UnknownOs(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub path: PathBuf,
}
