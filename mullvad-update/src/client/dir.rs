//! This provides a secure temp directory suitable for storing updates, with admin-only write access.

use std::path::PathBuf;

use anyhow::Context;

/// Name of subdirectory in the temp directory
const CACHE_DIRNAME: &str = "mullvad-updates";

/// This returns a directory suitable for storing updates, where only admins have write access.
///
/// This function is a bit racey, as the directory is created before being restricted.
/// This is acceptable as long as the checksum of each file is verified before being used.
pub async fn admin_temp_dir() -> anyhow::Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(CACHE_DIRNAME);

    #[cfg(windows)]
    {
        let dir_clone = temp_dir.clone();
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

        fs::create_dir_all(&temp_dir)
            .await
            .context("Failed to create cache directory")?;
        fs::set_permissions(&temp_dir, Permissions::from_mode(0o700))
            .await
            .context("Failed to set cache directory permissions")?;
    }

    Ok(temp_dir)
}
