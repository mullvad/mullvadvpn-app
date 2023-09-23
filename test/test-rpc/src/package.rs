use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(err_derive::Error, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed open file for writing")]
    OpenFile,

    #[error(display = "Failed to write downloaded file to disk")]
    WriteFile,

    #[error(display = "Failed to convert download to bytes")]
    ToBytes,

    #[error(display = "Failed to convert download to bytes")]
    RequestFailed,

    #[error(display = "Cannot parse version")]
    InvalidVersion,

    #[error(display = "Failed to run package installer")]
    RunApp,

    #[error(display = "Failed to create temporary uninstaller")]
    CreateTempUninstaller,

    #[error(
        display = "Installer or uninstaller failed due to an unknown error: {}",
        _0
    )]
    InstallerFailed(i32),

    #[error(display = "Installer or uninstaller failed due to a signal")]
    InstallerFailedSignal,

    #[error(display = "Unrecognized OS: {}", _0)]
    UnknownOs(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub path: PathBuf,
}
