use colored::Colorize;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    #[error(display = "Could not get standard output from runner")]
    StandardOutput,
    #[error(display = "Could not get mullvad app logs from runner")]
    Logs(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
    Error(String),
    Warning(String),
    Info(String),
    Other(String),
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Output::Error(s) => f.write_fmt(format_args!("{}", s.as_str().red())),
            Output::Warning(s) => f.write_fmt(format_args!("{}", s.as_str().yellow())),
            Output::Info(s) => f.write_fmt(format_args!("{}", s.as_str())),
            Output::Other(s) => f.write_fmt(format_args!("{}", s.as_str())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOutput {
    pub settings_json: Result<String>,
    pub log_files: Result<Vec<Result<LogFile>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFile {
    pub name: std::path::PathBuf,
    pub content: String,
}
