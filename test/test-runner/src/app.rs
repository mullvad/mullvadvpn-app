use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

use test_rpc::{AppTrace, Error};

/// Get the installed app version string
pub async fn version() -> Result<String, Error> {
    // The `mullvad` binary is seemingly not in PATH on Windows after upgrading the app..
    // So, as a workaround we use the absolute path instead.
    const MULLVAD_CLI_BIN: &str = if cfg!(target_os = "windows") {
        r"C:\Program Files\Mullvad VPN\resources\mullvad.exe"
    } else {
        "mullvad"
    };
    let version = tokio::process::Command::new(MULLVAD_CLI_BIN)
        .arg("--version")
        .output()
        .await
        .map_err(|e| Error::ServiceNotFound(e.to_string()))?;
    let version = String::from_utf8(version.stdout).map_err(|err| Error::Other(err.to_string()))?;
    // HACK: The output from `mullvad --version` includes the `mullvad-cli` binary name followed by
    // the version string. Simply remove the leading noise and get at the version string.
    let Some(version) = version.split_whitespace().nth(1) else {
        return Err(Error::Other(
            "Could not parse version number from `mullvad-cli --version`".to_string(),
        ));
    };
    let version = version.to_string();
    Ok(version)
}

#[cfg(target_os = "windows")]
pub fn find_traces() -> Result<Vec<AppTrace>, Error> {
    // TODO: Check GUI data
    // TODO: Check temp data
    // TODO: Check devices and drivers

    let settings_dir = mullvad_paths::get_default_settings_dir().map_err(|error| {
        log::error!("Failed to obtain system app data: {error}");
        Error::Syscall
    })?;

    let caches = find_cache_traces()?;
    let traces = vec![
        Path::new(r"C:\Program Files\Mullvad VPN"),
        // NOTE: This only works as of `499c06decda37dc639e5f` in the Mullvad app.
        // Older builds have no way of silently fully uninstalling the app.
        Path::new(r"C:\ProgramData\Mullvad VPN"),
        // NOTE: Works as of `4116ebc` (Mullvad app).
        &settings_dir,
        &caches,
    ];

    Ok(existing_paths(&traces))
}

#[cfg(target_os = "linux")]
pub fn find_traces() -> Result<Vec<AppTrace>, Error> {
    // TODO: Check GUI data
    // TODO: Check temp data

    let caches = find_cache_traces()?;
    let traces = vec![
        Path::new(r"/etc/mullvad-vpn/"),
        Path::new(r"/var/log/mullvad-vpn/"),
        &caches,
        Path::new(r"/opt/Mullvad VPN/"),
        // management interface socket
        Path::new(r"/var/run/mullvad-vpn"),
        // service unit config files
        Path::new(r"/usr/lib/systemd/system/mullvad-daemon.service"),
        Path::new(r"/usr/lib/systemd/system/mullvad-early-boot-blocking.service"),
        Path::new(r"/usr/bin/mullvad"),
        Path::new(r"/usr/bin/mullvad-daemon"),
        Path::new(r"/usr/bin/mullvad-exclude"),
        Path::new(r"/usr/bin/mullvad-problem-report"),
        Path::new(r"/usr/share/bash-completion/completions/mullvad"),
        Path::new(r"/usr/local/share/zsh/site-functions/_mullvad"),
        Path::new(r"/usr/share/fish/vendor_completions.d/mullvad.fish"),
    ];

    Ok(existing_paths(&traces))
}

pub fn find_cache_traces() -> Result<PathBuf, Error> {
    mullvad_paths::get_cache_dir().map_err(|error| Error::FileSystem(error.to_string()))
}

#[cfg(target_os = "macos")]
pub fn find_traces() -> Result<Vec<AppTrace>, Error> {
    // TODO: Check GUI data
    // TODO: Check temp data

    let caches = find_cache_traces()?;
    let traces = vec![
        Path::new(r"/Applications/Mullvad VPN.app/"),
        Path::new(r"/var/log/mullvad-vpn/"),
        &caches,
        // management interface socket
        Path::new(r"/var/run/mullvad-vpn"),
        // launch daemon
        Path::new(r"/Library/LaunchDaemons/net.mullvad.daemon.plist"),
        Path::new(r"/usr/local/bin/mullvad"),
        Path::new(r"/usr/local/bin/mullvad-problem-report"),
        // completions
        Path::new(r"/usr/local/share/zsh/site-functions/_mullvad"),
        Path::new(r"/opt/homebrew/share/fish/vendor_completions.d/mullvad.fish"),
        Path::new(r"/usr/local/share/fish/vendor_completions.d/mullvad.fish"),
    ];

    Ok(existing_paths(&traces))
}

/// Find all present app traces on the test runner.
fn existing_paths(paths: &[&Path]) -> Vec<AppTrace> {
    paths
        .iter()
        .filter(|&path| path.try_exists().is_ok_and(|exists| exists))
        .map(|path| AppTrace::Path(path.to_path_buf()))
        .collect()
}

pub async fn make_device_json_old() -> Result<(), Error> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const DEVICE_JSON_PATH: &str = "/etc/mullvad-vpn/device.json";
    #[cfg(target_os = "windows")]
    const DEVICE_JSON_PATH: &str =
        "C:\\Windows\\system32\\config\\systemprofile\\AppData\\Local\\Mullvad VPN\\device.json";
    let device_json = tokio::fs::read_to_string(DEVICE_JSON_PATH)
        .await
        .map_err(|e| Error::FileSystem(e.to_string()))?;

    let mut device_state: serde_json::Value =
        serde_json::from_str(&device_json).map_err(|e| Error::FileSerialization(e.to_string()))?;
    let created_ref: &mut serde_json::Value = device_state
        .get_mut("logged_in")
        .unwrap()
        .get_mut("device")
        .unwrap()
        .get_mut("wg_data")
        .unwrap()
        .get_mut("created")
        .unwrap();
    let created: DateTime<Utc> = serde_json::from_value(created_ref.clone()).unwrap();
    let created = created
        .checked_sub_signed(chrono::Duration::days(365))
        .unwrap();

    *created_ref = serde_json::to_value(created).unwrap();

    let device_json = serde_json::to_string(&device_state)
        .map_err(|e| Error::FileSerialization(e.to_string()))?;
    tokio::fs::write(DEVICE_JSON_PATH, device_json.as_bytes())
        .await
        .map_err(|e| Error::FileSystem(e.to_string()))?;

    Ok(())
}
