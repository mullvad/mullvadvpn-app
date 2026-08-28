use std::collections::HashMap;
use test_rpc::{meta::OsVersion, mullvad_daemon::Verbosity};

const PLIST_OVERRIDE_FILE: &str = "/Library/LaunchDaemons/net.mullvad.daemon.plist";

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
pub async fn restart_app() -> Result<(), test_rpc::Error> {
    stop_app().await?;
    start_app().await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
pub async fn stop_app() -> Result<(), test_rpc::Error> {
    set_launch_daemon_state(false).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
pub async fn start_app() -> Result<(), test_rpc::Error> {
    set_launch_daemon_state(true).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    Ok(())
}

#[expect(clippy::unused_async)]
pub async fn set_daemon_log_level(_verbosity_level: Verbosity) -> Result<(), test_rpc::Error> {
    // TODO: Not implemented
    log::warn!("Setting log level is not implemented on macOS");
    Ok(())
}

pub async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), test_rpc::Error> {
    tokio::task::spawn_blocking(|| {
        let mut parsed_plist = plist::Value::from_file(PLIST_OVERRIDE_FILE).map_err(|error| {
            test_rpc::Error::ServiceNotFound(format!("failed to parse plist: {error}"))
        })?;

        let mut vars = plist::Dictionary::new();
        for (k, v) in env {
            // Set environment globally (not for service) to prevent it from being lost on upgrade
            std::process::Command::new("launchctl")
                .arg("setenv")
                .args([&k, &v])
                .status()
                .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;
            vars.insert(k, plist::Value::String(v));
        }

        // Add permanent env var
        parsed_plist
            .as_dictionary_mut()
            .ok_or_else(|| test_rpc::Error::ServiceChange("plist missing dict".to_owned()))?
            .insert(
                "EnvironmentVariables".to_owned(),
                plist::Value::Dictionary(vars),
            );

        let daemon_plist = std::fs::File::create(PLIST_OVERRIDE_FILE)
            .map_err(|e| test_rpc::Error::ServiceChange(format!("failed to open plist: {e}")))?;

        parsed_plist
            .to_writer_xml(daemon_plist)
            .map_err(|e| test_rpc::Error::ServiceChange(format!("failed to replace plist: {e}")))?;

        Ok::<(), test_rpc::Error>(())
    })
    .await
    .unwrap()?;

    // Restart service
    set_launch_daemon_state(false).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    set_launch_daemon_state(true).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    Ok(())
}

async fn set_launch_daemon_state(on: bool) -> Result<(), test_rpc::Error> {
    let mut launchctl = tokio::process::Command::new("launchctl");
    if on {
        launchctl
            .args(["load", "-w", PLIST_OVERRIDE_FILE])
            .status()
            .await
            .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;
    } else {
        launchctl
            .args(["unload", "-w", PLIST_OVERRIDE_FILE])
            .status()
            .await
            .map_err(|e| test_rpc::Error::ServiceStop(e.to_string()))?;
    }
    Ok(())
}

pub async fn get_daemon_environment() -> Result<HashMap<String, String>, test_rpc::Error> {
    let plist = tokio::task::spawn_blocking(|| {
        let parsed_plist = plist::Value::from_file(PLIST_OVERRIDE_FILE).map_err(|error| {
            test_rpc::Error::ServiceNotFound(format!("failed to parse plist: {error}"))
        })?;

        Ok::<plist::Value, test_rpc::Error>(parsed_plist)
    })
    .await
    .map_err(test_rpc::Error::from_tokio_join_error)??;

    let plist_tree = plist
        .as_dictionary()
        .ok_or_else(|| test_rpc::Error::ServiceNotFound("plist missing dict".to_owned()))?;
    let Some(env_vars) = plist_tree.get("EnvironmentVariables") else {
        // `EnvironmentVariables` does not exist in plist file, so there are no env variables to
        // parse.
        return Ok(HashMap::new());
    };
    let env_vars = env_vars.as_dictionary().ok_or_else(|| {
        test_rpc::Error::ServiceNotFound("`EnvironmentVariables` is not a dict".to_owned())
    })?;

    let env = env_vars
        .clone()
        .into_iter()
        .filter_map(|(key, value)| Some((key, value.into_string()?)))
        .collect();

    Ok(env)
}

pub fn get_os_version() -> Result<OsVersion, test_rpc::Error> {
    use test_rpc::meta::MacosVersion;

    let version = talpid_platform_metadata::MacosVersion::new()
        .inspect_err(|error| {
            log::error!("Failed to obtain OS version: {error}");
        })
        .map_err(|_| test_rpc::Error::Syscall)?;

    Ok(OsVersion::Macos(MacosVersion {
        major: version.major_version(),
    }))
}
