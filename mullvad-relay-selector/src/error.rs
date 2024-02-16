//! Definition of relay selector errors

use mullvad_types::relay_constraints::MissingCustomBridgeSettings;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to open relay cache file")]
    OpenRelayCache(#[source] std::io::Error),

    #[error("Failed to write relay cache file to disk")]
    WriteRelayCache(#[source] std::io::Error),

    #[error("No relays matching current constraints")]
    NoRelay,

    #[error("No bridges matching current constraints")]
    NoBridge,

    #[error("No obfuscators matching current constraints")]
    NoObfuscator,

    #[error("Failure in serialization of the relay list")]
    Serialize(#[from] serde_json::Error),

    #[error("Downloader already shut down")]
    DownloaderShutDown,

    #[error("Invalid bridge settings")]
    InvalidBridgeSettings(#[from] MissingCustomBridgeSettings),
}
