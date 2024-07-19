use crate::config::{Architecture, OsType, PackageType, VmConfig};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

static VERSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\d{4}\.\d+(-beta\d+)?(-dev)?-([0-9a-z])+").unwrap());

#[derive(Debug, Clone)]
pub struct Manifest {
    pub app_package_path: PathBuf,
    pub app_package_to_upgrade_from_path: Option<PathBuf>,
    pub ui_e2e_tests_path: Option<PathBuf>,
}

/// Obtain app packages and their filenames
/// If it's a path, use the path.
/// If it corresponds to a file in packages/, use that package.
/// TODO: If it's a git tag or rev, download it.
pub fn get_app_manifest(
    config: &VmConfig,
    app_package: String,
    app_package_to_upgrade_from: Option<String>,
    package_folder: Option<PathBuf>,
) -> Result<Manifest> {
    let package_type = (config.os_type, config.package_type, config.architecture);

    let app_package_path = find_app(&app_package, false, package_type, package_folder.as_ref())?;
    log::info!("Current app: {}", app_package_path.display());

    let app_package_to_upgrade_from_path = app_package_to_upgrade_from
        .map(|app| find_app(&app, false, package_type, package_folder.as_ref()))
        .transpose()?;
    log::info!("Previous app: {app_package_to_upgrade_from_path:?}");

    let capture = VERSION_REGEX
        .captures(app_package_path.to_str().unwrap())
        .with_context(|| format!("Cannot parse version: {}", app_package_path.display()))?
        .get(0)
        .map(|c| c.as_str())
        .expect("Could not parse version from package name: {app_package}");

    let ui_e2e_tests_path = find_app(capture, true, package_type, package_folder.as_ref()).ok();
    log::info!("GUI e2e test binary: {ui_e2e_tests_path:?}");

    Ok(Manifest {
        app_package_path,
        app_package_to_upgrade_from_path,
        ui_e2e_tests_path,
    })
}

fn find_app(
    app: &str,
    e2e_bin: bool,
    package_type: (OsType, Option<PackageType>, Option<Architecture>),
    package_folder: Option<&PathBuf>,
) -> Result<PathBuf> {
    // If it's a path, use that path
    let app_path = Path::new(app);
    if app_path.is_file() {
        // TODO: Copy to packages?
        return Ok(app_path.to_path_buf());
    }

    let mut app = app.to_owned();
    app.make_ascii_lowercase();

    let current_dir = std::env::current_dir().expect("Unable to get current directory");
    let packages_dir = package_folder.unwrap_or(&current_dir);
    std::fs::create_dir_all(packages_dir)?;
    let mut dir = std::fs::read_dir(packages_dir.clone()).context("Failed to list packages")?;

    let mut matches = vec![];

    while let Some(Ok(entry)) = dir.next() {
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
