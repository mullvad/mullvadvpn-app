//! This provides a secure directory suitable for storing updates, with admin-only write access.

use std::path::PathBuf;
use tokio::fs;

use anyhow::Context;

/// Name of subdirectory in the cache directory
const CACHE_DIRNAME: &str = "updates";

/// This returns a directory suitable for storing updates. Only admins have write access.
///
/// This function is a bit racey, as the directory is created before being restricted.
/// This is acceptable as long as the checksum of each file is verified before being used.
pub async fn update_directory() -> anyhow::Result<PathBuf> {
    let dir = tokio::task::spawn_blocking(|| mullvad_paths::cache_dir())
        .await
        .unwrap()?
        .join(CACHE_DIRNAME);

    #[cfg(windows)]
    {
        let dir_clone = dir.clone();
        tokio::task::spawn_blocking(move || {
            mullvad_paths::windows::create_privileged_directory(&dir_clone)
        })
        .await
        .unwrap()
        .context("Failed to create cache directory")?;
    }

    #[cfg(unix)]
    {
        use std::{fs::Permissions, os::unix::fs::PermissionsExt};
        use tokio::fs;

        fs::create_dir_all(&dir)
            .await
            .context("Failed to create cache directory")?;
        fs::set_permissions(&dir, Permissions::from_mode(0o700))
            .await
            .context("Failed to set cache directory permissions")?;
    }

    Ok(dir)
}

/// Remove all files from the update directory
pub async fn cleanup_update_directory() -> anyhow::Result<()> {

    let dir = update_directory().await?;

    // It's fine to remove the directory in its entirety, since `update_directory` recreates it.
    fs::remove_dir_all(dir).await?;

    Ok(())
}
