use std::io;

pub mod check;
pub mod router;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open app version cache file for reading")]
    ReadVersionCache(#[source] io::Error),

    #[error("Failed to open app version cache file for writing")]
    WriteVersionCache(#[source] io::Error),

    #[error("Failure in serialization of the version info")]
    Serialize(#[source] serde_json::Error),

    #[error("Failure in deserialization of the version info")]
    Deserialize(#[source] serde_json::Error),

    #[error("Failed to check the latest app version")]
    Download(#[source] mullvad_api::rest::Error),

    #[error("API availability check failed")]
    ApiCheck(#[source] mullvad_api::availability::Error),

    #[error("Clearing version check cache due to a version mismatch")]
    CacheVersionMismatch,

    #[error("Version updater is down")]
    VersionUpdaterDown,

    #[error("Version router is down")]
    VersionRouterClosed,

    #[error("Version cache update was aborted")]
    UpdateAborted,
}

/// Contains the date of the git commit this was built from
pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub fn is_beta_version() -> bool {
    mullvad_version::VERSION.contains("beta")
}

pub fn is_dev_version() -> bool {
    mullvad_version::VERSION.contains("dev")
}

pub fn log_version() {
    log::info!(
        "Starting {} - {} {}",
        env!("CARGO_PKG_NAME"),
        mullvad_version::VERSION,
        COMMIT_DATE,
    )
}
