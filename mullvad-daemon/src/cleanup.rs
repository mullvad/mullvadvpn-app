#![cfg(not(target_os = "android"))]

use std::path::Path;
use tokio::{fs, io};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get path")]
    Path(#[source] mullvad_paths::Error),

    #[error("Failed to remove directory {0}")]
    RemoveDir(String, #[source] io::Error),

    #[cfg(not(target_os = "windows"))]
    #[error("Failed to create directory {0}")]
    CreateDir(String, #[source] io::Error),

    #[cfg(target_os = "windows")]
    #[error("Failed to get file type info")]
    FileType(#[source] io::Error),

    #[cfg(target_os = "windows")]
    #[error("Failed to get dir entry")]
    FileEntry(#[source] io::Error),

    #[cfg(target_os = "windows")]
    #[error("Failed to read dir entries")]
    ReadDir(#[source] io::Error),
}

pub async fn clear_directories() -> Result<(), Error> {
    clear_log_directory().await?;
    clear_cache_directory().await
}

async fn clear_log_directory() -> Result<(), Error> {
    let log_dir = mullvad_paths::get_log_dir().map_err(Error::Path)?;
    clear_directory(&log_dir).await
}

async fn clear_cache_directory() -> Result<(), Error> {
    let cache_dir = mullvad_paths::cache_dir().map_err(Error::Path)?;
    clear_directory(&cache_dir).await
}

async fn clear_directory(path: &Path) -> Result<(), Error> {
    #[cfg(not(target_os = "windows"))]
    {
        fs::remove_dir_all(path)
            .await
            .map_err(|e| Error::RemoveDir(path.display().to_string(), e))?;
        fs::create_dir_all(path)
            .await
            .map_err(|e| Error::CreateDir(path.display().to_string(), e))
    }
    #[cfg(target_os = "windows")]
    {
        let mut dir = fs::read_dir(&path).await.map_err(Error::ReadDir)?;

        let mut result = Ok(());

        while let Some(entry) = dir.next_entry().await.map_err(Error::FileEntry)? {
            let entry_type = match entry.file_type().await {
                Ok(entry_type) => entry_type,
                Err(error) => {
                    result = result.and(Err(Error::FileType(error)));
                    continue;
                }
            };

            let removal = if entry_type.is_file() || entry_type.is_symlink() {
                fs::remove_file(entry.path()).await
            } else {
                fs::remove_dir_all(entry.path()).await
            };
            result = result
                .and(removal.map_err(|e| Error::RemoveDir(entry.path().display().to_string(), e)));
        }
        result
    }
}
