use crate::config::{Architecture, OsType, PackageType, VmConfig};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs;

static VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\d{4}\.\d+(-beta\d+)?(-dev)?-([0-9a-z])+").unwrap());

#[derive(Debug, Clone)]
pub struct Manifest {
    pub current_app_path: PathBuf,
    pub previous_app_path: PathBuf,
    pub ui_e2e_tests_path: PathBuf,
}

/// Obtain app packages and their filenames
/// If it's a path, use the path.
/// If it corresponds to a file in packages/, use that package.
/// TODO: If it's a git tag or rev, download it.
pub async fn get_app_manifest(
    config: &VmConfig,
    current_app: String,
    previous_app: String,
) -> Result<Manifest> {
    let package_type = (config.os_type, config.package_type, config.architecture);

    let current_app_path = find_app(&current_app, false, package_type).await?;
    log::info!("Current app: {}", current_app_path.display());

    let previous_app_path = find_app(&previous_app, false, package_type).await?;
    log::info!("Previous app: {}", previous_app_path.display());

    let capture = VERSION_REGEX
        .captures(current_app_path.to_str().unwrap())
        .with_context(|| format!("Cannot parse version: {}", current_app_path.display()))?
        .get(0)
        .map(|c| c.as_str())
        .expect("Could not parse version from package name: {current_app}");

    let ui_e2e_tests_path = find_app(capture, true, package_type).await?;
    log::info!("Runner executable: {}", ui_e2e_tests_path.display());

    Ok(Manifest {
        current_app_path,
        previous_app_path,
        ui_e2e_tests_path,
    })
}

async fn find_app(
    app: &str,
    e2e_bin: bool,
    package_type: (OsType, Option<PackageType>, Option<Architecture>),
) -> Result<PathBuf> {
    // If it's a path, use that path
    let app_path = Path::new(app);
    if app_path.is_file() {
        // TODO: Copy to packages?
        return Ok(app_path.to_path_buf());
    }

    let mut app = app.to_owned();
    app.make_ascii_lowercase();

    let packages_dir = dirs::cache_dir()
        .context("Could not find cache directory")?
        .join("mullvad-test")
        .join("packages");
    fs::create_dir_all(&packages_dir).await?;
    let mut dir = fs::read_dir(packages_dir.clone())
        .await
        .context("Failed to list packages")?;

    let mut matches = vec![];

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Filter out irrelevant platforms
        if !e2e_bin {
            let ext = get_ext(package_type);

            // Skip file if wrong file extension
            if !path
                .extension()
                .map(|m_ext| m_ext.eq_ignore_ascii_case(ext))
                .unwrap_or(false)
            {
                continue;
            }
        }

        let mut u8_path = path.as_os_str().to_string_lossy().into_owned();
        u8_path.make_ascii_lowercase();

        // Skip non-UI-e2e binaries or vice versa
        if e2e_bin ^ u8_path.contains("app-e2e-tests") {
            continue;
        }

        // Filter out irrelevant platforms
        if e2e_bin && !u8_path.contains(get_os_name(package_type)) {
            continue;
        }

        // Skip file if it doesn't match the architecture
        if let Some(arch) = package_type.2 {
            // Skip for non-e2e bin on non-Linux, because there's only one package
            if (e2e_bin || package_type.0 == OsType::Linux)
                && !arch.get_identifiers().iter().any(|id| u8_path.contains(id))
            {
                continue;
            }
        }

        if u8_path.contains(&app) {
            matches.push(path);
        }
    }

    // TODO: Search for package in git repository if not found

    // Take the shortest match
    matches.sort_unstable_by_key(|path| path.as_os_str().len());
    matches.into_iter().next().context(if e2e_bin {
        format!(
            "Could not find UI/e2e test for package: {app}.\n\
         Expecting a binary named like `app-e2e-tests-{app}_ARCH` to exist in {package_dir}/\n\
         Example ARCH: `amd64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`",
            package_dir = packages_dir.display()
        )
    } else {
        format!("Could not find package for app: {app}")
    })
}

fn get_ext(package_type: (OsType, Option<PackageType>, Option<Architecture>)) -> &'static str {
    match package_type.0 {
        OsType::Windows => "exe",
        OsType::Macos => "pkg",
        OsType::Linux => match package_type.1.expect("must specify package type") {
            PackageType::Deb => "deb",
            PackageType::Rpm => "rpm",
        },
    }
}

fn get_os_name(package_type: (OsType, Option<PackageType>, Option<Architecture>)) -> &'static str {
    match package_type.0 {
        OsType::Windows => "windows",
        OsType::Macos => "apple",
        OsType::Linux => "linux",
    }
}
