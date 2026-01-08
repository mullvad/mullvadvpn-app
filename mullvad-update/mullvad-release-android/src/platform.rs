//! Types for handling per-platform metadata

use anyhow::{Context, anyhow, bail};
use mullvad_update::api::{HttpVersionInfoProvider, MetaRepositoryPlatform};
use mullvad_update::format::Architecture;
use mullvad_update::format::release::Release;
use mullvad_update::format::response::AndroidReleases;
use mullvad_update::version::{MIN_VERIFY_METADATA_VERSION, VersionInfo, VersionParameters};
use mullvad_update::version::{Response, Rollout};
use std::{cmp::Ordering, path::PathBuf};
use tokio::{fs, io};

use crate::{
    get_data_dir,
    io_util::{create_dir_and_write, wait_for_confirm},
};

/// Output used by `Platform::query_latest`
#[derive(serde::Serialize)]
pub struct VersionQueryOutput {
    /// Stable version info
    pub stable: mullvad_version::Version,
    /// Beta version info (if available and newer than `stable`).
    /// If latest stable version is newer, this will be `None`.
    pub beta: Option<mullvad_version::Version>,
}

/// Path to WIP file in `work/` for this platform
pub fn work_path() -> PathBuf {
    get_data_dir().join("work").join("android.json")
}

/// Pull latest metadata from repository and store it in `work/`
pub async fn pull(assume_yes: bool) -> anyhow::Result<()> {
    let platform = MetaRepositoryPlatform::Android;

    println!("Pulling Android metadata from {}...", platform.url());

    let releases = HttpVersionInfoProvider::get_android_releases()
        .await
        .context("Failed to retrieve versions")?;

    let json =
        serde_json::to_string_pretty(&releases).context("Failed to serialize updated metadata")?;

    let work_path = work_path();

    // Require confirmation if a file exists
    if !assume_yes && work_path.exists() {
        let msg = format!(
            "This will replace the existing file at {}. Continue?",
            work_path.display()
        );
        if !wait_for_confirm(&msg).await {
            bail!("Aborted update");
        }
    }

    println!("Writing metadata to {}", work_path.display());

    create_dir_and_write(&work_path, &json).await?;

    println!("Updated {}", work_path.display());
    Ok(())
}

/// Add release to platform in `work/`
pub async fn add_release(version: &mullvad_version::Version) -> anyhow::Result<()> {
    // Fetch WIP versions and verify that release does not exist
    let work_path = work_path();
    println!("Adding {version} from {}", work_path.display());

    let mut work_response = read_work().await?;
    if work_response
        .releases
        .iter()
        .any(|release| &release.version == version)
    {
        // If it doesn't exist, treat as success
        bail!("Version {version} already exists");
    }

    // Make release
    let new_release = Release {
        changelog: "".to_owned(),
        version: version.clone(),
        installers: vec![],
        rollout: Rollout::complete(),
    };

    println!("- {}", &new_release.version);

    work_response.releases.push(new_release);

    let json = serde_json::to_string_pretty(&work_response)
        .context("Failed to serialize updated metadata")?;
    create_dir_and_write(&work_path, &json).await?;

    println!("Added {version} to {}", work_path.display());

    Ok(())
}

/// List releases for platforms in `work/`
pub async fn list_releases() -> anyhow::Result<()> {
    let work_path = work_path();
    println!("Releases for file {}", work_path.display());

    let mut response = read_work().await?;

    if response.releases.is_empty() {
        println!("No releases");
        return Ok(());
    }

    response
        .releases
        .sort_by(|a, b| b.version.partial_cmp(&a.version).unwrap_or(Ordering::Equal));

    for release in &response.releases {
        println!("- {}", &release.version);
    }
    Ok(())
}

/// Remove version/release in `work/`
pub async fn remove_release(version: &mullvad_version::Version) -> anyhow::Result<()> {
    let work_path = work_path();
    println!("Removing {version} from {}", work_path.display());

    let mut work_response = read_work().await?;

    let Some(found_release_ind) = work_response
        .releases
        .iter()
        .position(|release| &release.version == version)
    else {
        // If it doesn't exist, treat as success
        return Ok(());
    };

    let removed_release = work_response.releases.swap_remove(found_release_ind);

    println!("- {}", &removed_release.version);

    let json = serde_json::to_string_pretty(&work_response)
        .context("Failed to serialize updated metadata")?;
    create_dir_and_write(&work_path, &json).await?;

    println!("Removed {version} in {}", work_path.display());

    Ok(())
}

/// Return the latest release for platforms in `work/`
pub async fn query_latest() -> anyhow::Result<VersionQueryOutput> {
    let response = read_work().await?;

    let version_info =
        VersionInfo::find_latest_versions(response.releases.into_iter(), |release| {
            &release.version
        })?;

    Ok(VersionQueryOutput {
        stable: version_info.0.version,
        beta: version_info.1.map(|release| release.version),
    })
}

/// Reads the metadata for `platform` in the work directory.
/// If the file doesn't exist, this returns a new, empty response.
async fn read_work() -> anyhow::Result<AndroidReleases> {
    let work_path = work_path();
    let bytes = match fs::read(&work_path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            // Return empty response
            return Ok(AndroidReleases::default());
        }
        Err(error) => bail!("Failed to read {}: {error}", work_path.display()),
    };
    serde_json::from_slice(&bytes)
        .with_context(|| anyhow!("Failed to parse {}", work_path.display()))
}
