#![cfg(not(target_os = "android"))]

use std::path::Path;
use tokio::{fs, io};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to get path")]
    PathError(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to remove directory {}", _0)]
    RemoveDirError(String, #[error(source)] io::Error),

    #[cfg(not(target_os = "windows"))]
    #[error(display = "Failed to create directory {}", _0)]
    CreateDirError(String, #[error(source)] io::Error),

    #[cfg(target_os = "windows")]
    #[error(display = "Failed to get file type info")]
    FileTypeError(#[error(source)] io::Error),

    #[cfg(target_os = "windows")]
    #[error(display = "Failed to get dir entry")]
    FileEntryError(#[error(source)] io::Error),

    #[cfg(target_os = "windows")]
    #[error(display = "Failed to read dir entries")]
    ReadDirError(#[error(source)] io::Error),
}

pub async fn clear_directories() -> Result<(), Error> {
    clear_log_directory().await?;
    clear_cache_directory().await
}

async fn clear_log_directory() -> Result<(), Error> {
    let log_dir = mullvad_paths::get_log_dir().map_err(Error::PathError)?;
    clear_directory(&log_dir).await
}

async fn clear_cache_directory() -> Result<(), Error> {
    let cache_dir = mullvad_paths::cache_dir().map_err(Error::PathError)?;
    clear_directory(&cache_dir).await
}

async fn clear_directory(path: &Path) -> Result<(), Error> {
    #[cfg(not(target_os = "windows"))]
    {
        fs::remove_dir_all(path)
            .await
            .map_err(|e| Error::RemoveDirError(path.display().to_string(), e))?;
        fs::create_dir_all(path)
            .await
            .map_err(|e| Error::CreateDirError(path.display().to_string(), e))
    }
    #[cfg(target_os = "windows")]
    {
        let mut dir = fs::read_dir(&path).await.map_err(Error::ReadDirError)?;

        let mut result = Ok(());

        while let Some(entry) = dir.next_entry().await.map_err(Error::FileEntryError)? {
            let entry_type = match entry.file_type().await {
                Ok(entry_type) => entry_type,
                Err(error) => {
                    result = result.and(Err(Error::FileTypeError(error)));
                    continue;
                }
            };

            let removal = if entry_type.is_file() || entry_type.is_symlink() {
                fs::remove_file(entry.path()).await
            } else {
                fs::remove_dir_all(entry.path()).await
            };
            result = result.and(
                removal.map_err(|e| Error::RemoveDirError(entry.path().display().to_string(), e)),
            );
        }
        result
    }
}
