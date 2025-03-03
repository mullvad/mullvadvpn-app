//! Creates a temporary directory for the installer.
//!
//! # Windows
//!
//! Since the Windows downloader runs as admin, we can use a persistent directory and prevent
//! non-admins from accessing it.
//!
//! # macOS
//!
//! The downloader does not run as a privileged user, so we store downloads in a temporary
//! directory.
//!
//! This is vulnerable to TOCTOU, ie replacing the file after its hash has been verified, but only
//! by the current user. Using a random directory name mitigates this issue.

use anyhow::Context;
use async_trait::async_trait;
use std::path::PathBuf;

/// Provide a directory to use for [AppDownloader]
#[async_trait]
pub trait DirectoryProvider {
    /// Provide a directory to use for [AppDownloader]
    async fn create_download_dir() -> anyhow::Result<PathBuf>;
}

/// See [module-level](self) docs.
pub struct TempDirProvider;

#[async_trait]
impl DirectoryProvider for TempDirProvider {
    /// Create a locked-down directory to store downloads in
    async fn create_download_dir() -> anyhow::Result<PathBuf> {
        #[cfg(windows)]
        {
            admin_temp_dir().await
        }

        #[cfg(target_os = "macos")]
        {
            temp_dir().await
        }
    }
}

/// This returns a directory where only admins have write access.
///
/// This function is a bit racey, as the directory is created before being restricted.
/// This is acceptable as long as the checksum of each file is verified before being used.
#[cfg(windows)]
async fn admin_temp_dir() -> anyhow::Result<PathBuf> {
    /// Name of subdirectory in the temp directory
    const CACHE_DIRNAME: &str = "mullvad-updates";

    let temp_dir = std::env::temp_dir().join(CACHE_DIRNAME);

    let dir_clone = temp_dir.clone();
    tokio::task::spawn_blocking(move || {
        mullvad_paths::windows::create_privileged_directory(&dir_clone)
    })
    .await
    .unwrap()
    .context("Failed to create cache directory")?;

    Ok(temp_dir)
}

#[cfg(target_os = "macos")]
async fn temp_dir() -> anyhow::Result<PathBuf> {
    use rand::{distributions::Alphanumeric, Rng};
    use std::{fs::Permissions, os::unix::fs::PermissionsExt};
    use tokio::fs;

    // Randomly generate a directory name
    let dir_name: String = (0..10)
        .map(|_| rand::thread_rng().sample(Alphanumeric) as char)
        .collect();
    let temp_dir = std::env::temp_dir().join(dir_name);

    fs::create_dir_all(&temp_dir)
        .await
        .context("Failed to create cache directory")?;
    fs::set_permissions(&temp_dir, Permissions::from_mode(0o700))
        .await
        .context("Failed to set cache directory permissions")?;

    Ok(temp_dir)
}
