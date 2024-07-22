use crate::config::{Architecture, OsType, PackageType, VmConfig};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

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
    log::info!("App package: {}", app_package_path.display());

    let app_package_to_upgrade_from_path = app_package_to_upgrade_from
        .map(|app| find_app(&app, false, package_type, package_folder.as_ref()))
        .transpose()?;
    log::info!("App package to upgrade from: {app_package_to_upgrade_from_path:?}");

    // Automatically try to find the UI e2e tests based on the app package

    // Search the specified package folder, or same folder as the app package if missing
    let ui_e2e_package_folder = package_folder.unwrap_or(
        app_package_path
            .parent()
            .expect("Path to app package should have parent")
            .into(),
    );
    let capture = get_version_from_path(&app_package_path)?;

    let ui_e2e_tests_path =
        find_app(capture, true, package_type, Some(&ui_e2e_package_folder)).ok();
    log::info!("GUI e2e test binary: {ui_e2e_tests_path:?}");

    Ok(Manifest {
        app_package_path,
        app_package_to_upgrade_from_path,
        ui_e2e_tests_path,
    })
}

fn get_version_from_path(app_package_path: &Path) -> Result<&str, anyhow::Error> {
    static VERSION_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\d{4}\.\d+((-beta\d+)?(-dev)?-([0-9a-z])+)?").unwrap());

    VERSION_REGEX
        .captures(app_package_path.to_str().unwrap())
        .with_context(|| format!("Cannot parse version: {}", app_package_path.display()))?
        .get(0)
        .map(|c| c.as_str())
        .context("Could not parse version from package name: {app_package}")
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
    let dir = std::fs::read_dir(packages_dir.clone()).context("Failed to list packages")?;

    dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|entry| entry.is_file())
        .filter(|path| {
            e2e_bin ||
            path
                .extension()
                .map(|m_ext| m_ext.eq_ignore_ascii_case(get_ext(package_type)))
                .unwrap_or(false)
        }) // Filter out irrelevant platforms
        .map(|path| {
            let u8_path = path.as_os_str().to_string_lossy().to_ascii_lowercase();
            (path, u8_path)
        })
        .filter(|(_path, u8_path)| !(e2e_bin ^ u8_path.contains("app-e2e-tests"))) // Skip non-UI-e2e binaries or vice versa
        .filter(|(_path, u8_path)| !e2e_bin || u8_path.contains(get_os_name(package_type))) // Filter out irrelevant platforms
        .filter(|(_path, u8_path)| {
            let linux = e2e_bin || package_type.0 == OsType::Linux;
            let matching_ident = package_type.2.map(|arch| arch.get_identifiers().iter().any(|id| u8_path.contains(id))).unwrap_or(true);
            // Skip for non-Linux, because there's only one package
            !linux || matching_ident
        }) // Skip file if it doesn't match the architecture
        .find(|(_path, u8_path)| u8_path.contains(&app)) //  Find match
        .map(|(path, _)| path).context(if e2e_bin {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_regex() {
        let path = Path::new("../some/path/MullvadVPN-2024.4-beta1-dev-f7df8e_amd64.deb");
        let capture = get_version_from_path(path).unwrap();
        assert_eq!(capture, "2024.4-beta1-dev-f7df8e");

        let path = Path::new("../some/path/MullvadVPN-2024.4-beta1-f7df8e_amd64.deb");
        let capture = get_version_from_path(path).unwrap();
        assert_eq!(capture, "2024.4-beta1-f7df8e");

        let path = Path::new("../some/path/MullvadVPN-2024.4-dev-f7df8e_amd64.deb");
        let capture = get_version_from_path(path).unwrap();
        assert_eq!(capture, "2024.4-dev-f7df8e");

        let path = Path::new("../some/path/MullvadVPN-2024.3_amd64.deb");
        let capture = get_version_from_path(path).unwrap();
        assert_eq!(capture, "2024.3");
    }
}
