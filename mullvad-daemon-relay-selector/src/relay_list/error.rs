//! Error type and kinds for relay_list module.

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Downloader already shut down")]
    DownloaderShutdown,

    #[error("Failed to open relay cache file")]
    OpenRelayCache(#[source] std::io::Error),

    #[error("Failed to write relay cache file to disk")]
    WriteRelayCache(#[source] std::io::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
}
