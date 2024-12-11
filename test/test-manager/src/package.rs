use crate::config::{Architecture, OsType, PackageType, VmConfig};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[derive(Debug, Clone)]
pub struct Manifest {
    pub app_package_path: PathBuf,
    pub app_package_to_upgrade_from_path: Option<PathBuf>,
    pub gui_package_path: Option<PathBuf>,
}

/// Basic metadata about the test runner target platform such as OS, architecture and package
/// manager.
#[derive(Debug, Clone, Copy)]
pub enum TargetInfo {
    Windows {
        arch: Architecture,
    },
    Macos {
        arch: Architecture,
    },
    Linux {
        arch: Architecture,
        package_type: PackageType,
    },
}

/// Obtain app packages and their filenames
/// If it's a path, use the path.
/// If it corresponds to a file in packages/, use that package.
/// TODO: If it's a git tag or rev, download it.
pub fn get_app_manifest(
    test_runner: TargetInfo,
    app_package: String,
    app_package_to_upgrade_from: Option<String>,
    gui_package: Option<String>,
    package_dir: Option<PathBuf>,
) -> Result<Manifest> {
    let app_package_path = find_app(&app_package, false, test_runner, package_dir.as_ref())?;
    log::info!("App package: {}", app_package_path.display());

    let app_package_to_upgrade_from_path = app_package_to_upgrade_from
        .map(|app| find_app(&app, false, test_runner, package_dir.as_ref()))
        .transpose()?;
    log::info!("App package to upgrade from: {app_package_to_upgrade_from_path:?}");

    // Automatically try to find the UI e2e tests based on the app package

    // Search the specified package folder, or same folder as the app package if missing
    let ui_e2e_package_dir = package_dir.unwrap_or(
        app_package_path
            .parent()
            .expect("Path to app package should have parent")
            .into(),
    );

    let app_version = get_version_from_path(&app_package_path)?;
    let gui_package_path = find_app(
        match &gui_package {
            Some(gui_package) => gui_package,
            None => &app_version,
        },
        true,
        test_runner,
        Some(&ui_e2e_package_dir),
    );

    // Don't allow the UI/e2e test binary to missing if it's flag was specified
    let gui_package_path = match gui_package {
        Some(_) => Some(gui_package_path.context("Could not find specified UI/e2e test binary")?),
        None => gui_package_path.ok(),
    };

    log::info!("GUI e2e test binary: {gui_package_path:?}");

    Ok(Manifest {
        app_package_path,
        app_package_to_upgrade_from_path,
        gui_package_path,
    })
}

pub fn get_version_from_path(app_package_path: &Path) -> Result<String, anyhow::Error> {
    static VERSION_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\d{4}\.\d+((-beta\d+)?(-dev)?-([0-9a-z])+)?").unwrap());

    VERSION_REGEX
        .captures(app_package_path.to_str().unwrap())
        .with_context(|| format!("Cannot parse version: {}", app_package_path.display()))?
        .get(0)
        .map(|c| c.as_str().to_owned())
        .context("Could not parse version from package name: {app_package}")
}

fn find_app(
    app: &str,
    e2e_bin: bool,
    test_runner: TargetInfo,
    package_dir: Option<&PathBuf>,
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
    let package_dir = package_dir.unwrap_or(&current_dir);
    std::fs::create_dir_all(package_dir)?;
    let dir = std::fs::read_dir(package_dir.clone()).context("Failed to list packages")?;

    dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|entry| entry.is_file())
        .filter(|path| {
            e2e_bin ||
            path
                .extension()
                .map(|m_ext| m_ext.eq_ignore_ascii_case(test_runner.get_ext()))
                .unwrap_or(false)
        }) // Filter out irrelevant platforms
        .map(|path| {
            let u8_path = path.as_os_str().to_string_lossy().to_ascii_lowercase();
            (path, u8_path)
        })
        .filter(|(_path, u8_path)| !(e2e_bin ^ u8_path.contains("app-e2e-tests"))) // Skip non-UI-e2e binaries or vice versa
        .filter(|(_path, u8_path)| !e2e_bin || u8_path.contains(test_runner.get_os_name())) // Filter out irrelevant platforms
        .filter(|(_path, u8_path)| {
            let linux = e2e_bin || test_runner.is_linux();
            let matching_ident = test_runner.get_identifiers().any(|id| u8_path.contains(id));
            // Skip for non-Linux, because there's only one package
            !linux || matching_ident
        }) // Skip file if it doesn't match the architecture
        .sorted_unstable_by_key(|(_path, u8_path)| u8_path.len())
        .find(|(_path, u8_path)| u8_path.contains(&app)) //  Find match
        .map(|(path, _)| path)
        .with_context(|| format!("Directory searched: {}", package_dir.display()))
        .with_context(|| if e2e_bin {
            format!(
                "Could not find UI/e2e test for package: {app}.\n\
                Expecting a binary named like `app-e2e-tests-{app}_ARCH` to exist in {package_dir}/\n\
                Example ARCH: `amd64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`",
                package_dir = package_dir.display()
            )
        } else {
            format!("Could not find package for app: {app}")
        })
}

impl TargetInfo {
    const fn is_linux(self) -> bool {
        matches!(self, TargetInfo::Linux { .. })
    }

    const fn get_ext(self) -> &'static str {
        match self {
            TargetInfo::Windows { .. } => "exe",
            TargetInfo::Macos { .. } => "pkg",
            TargetInfo::Linux { package_type, .. } => match package_type {
                PackageType::Deb => "deb",
                PackageType::Rpm => "rpm",
            },
        }
    }

    const fn get_os_name(self) -> &'static str {
        match self {
            TargetInfo::Windows { .. } => "windows",
            TargetInfo::Macos { .. } => "apple",
            TargetInfo::Linux { .. } => "linux",
        }
    }

    fn get_identifiers(self) -> impl Iterator<Item = &'static str> {
        match self {
            TargetInfo::Windows { arch }
            | TargetInfo::Macos { arch }
            | TargetInfo::Linux { arch, .. } => arch.get_identifiers().into_iter(),
        }
    }
}

impl TryFrom<&VmConfig> for TargetInfo {
    type Error = anyhow::Error;

    fn try_from(config: &VmConfig) -> std::result::Result<Self, Self::Error> {
        let target_info = match config.os_type {
            OsType::Windows => TargetInfo::Windows {
                arch: config.architecture,
            },
            OsType::Macos => TargetInfo::Macos {
                arch: config.architecture,
            },
            OsType::Linux => {
                let Some(package_type) = config.package_type else {
                    bail!("Linux VM configuration did not specify any package type (Deb|Rpm)!");
                };
                TargetInfo::Linux {
                    arch: config.architecture,
                    package_type,
                }
            }
        };
        Ok(target_info)
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
