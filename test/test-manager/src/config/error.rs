use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not find config dir")]
    FindConfigDir,
    #[error("Could not create config dir")]
    CreateConfigDir(#[source] io::Error),
    #[error("Failed to read config")]
    Read(#[source] io::Error),
    #[error("Failed to parse config")]
    InvalidConfig(#[from] serde_json::Error),
    #[error("Failed to write config")]
    Write(#[source] io::Error),
}
