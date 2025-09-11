use std::collections::HashMap;
use test_rpc::{meta::OsVersion, mullvad_daemon::Verbosity};

const SYSTEMD_OVERRIDE_FILE: &str = "/etc/systemd/system/mullvad-daemon.service.d/override.conf";

pub async fn set_daemon_log_level(verbosity_level: Verbosity) -> Result<(), test_rpc::Error> {
    use tokio::io::AsyncWriteExt;
    log::debug!("Setting log level");

    let verbosity = match verbosity_level {
        Verbosity::Info => "",
        Verbosity::Debug => "-v",
        Verbosity::Trace => "-vv",
    };
    let systemd_service_file_content = format!(
        r#"[Service]
ExecStart=
ExecStart=/usr/bin/mullvad-daemon --disable-stdout-timestamps {verbosity}"#
    );

    let override_path = std::path::Path::new(SYSTEMD_OVERRIDE_FILE);
    if let Some(parent) = override_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(override_path)
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    file.write_all(systemd_service_file_content.as_bytes())
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    tokio::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;

    restart_app().await?;
    Ok(())
}

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
pub async fn restart_app() -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["restart", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;
    wait_for_service_state(ServiceState::Running).await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
pub async fn stop_app() -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["stop", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStop(e.to_string()))?;
    wait_for_service_state(ServiceState::Inactive).await?;

    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
pub async fn start_app() -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["start", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;
    wait_for_service_state(ServiceState::Running).await?;
    Ok(())
}

#[derive(Debug)]
struct EnvVar {
    var: String,
    value: String,
}

impl EnvVar {
    fn from_systemd_string(s: &str) -> Result<Self, &'static str> {
        // Here, we are only concerned with parsing a line that starts with "Environment".
        let error = "Failed to parse systemd env-config";
        let mut input = s.trim().split('=');
        let pre = input.next().ok_or(error)?;
        match pre {
            "Environment" => {
                // Process the input just a bit more - remove the leading and trailing quote (").
                let var = input
                    .next()
                    .ok_or(error)?
                    .trim_start_matches('"')
                    .to_string();
                let value = input.next().ok_or(error)?.trim_end_matches('"').to_string();
                Ok(EnvVar { var, value })
            }
            _ => Err(error),
        }
    }

    fn to_systemd_string(&self) -> String {
        format!(
            "Environment=\"{key}={value}\"",
            key = self.var,
            value = self.value
        )
    }
}

pub async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), test_rpc::Error> {
    use std::{fmt::Write, ops::Not};

    let mut override_content = String::new();
    override_content.push_str("[Service]\n");

    for env_var in env
        .into_iter()
        .map(|(var, value)| EnvVar { var, value })
        .map(|env_var| env_var.to_systemd_string())
    {
        writeln!(&mut override_content, "{env_var}")
            .map_err(|err| test_rpc::Error::ServiceChange(err.to_string()))?;
    }

    let override_path = std::path::Path::new(SYSTEMD_OVERRIDE_FILE);
    if let Some(parent) = override_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;
    }

    tokio::fs::write(override_path, override_content)
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    if tokio::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Io(e.to_string()))?
        .success()
        .not()
    {
        return Err(test_rpc::Error::ServiceChange(
            "Daemon service could not be reloaded".to_owned(),
        ));
    };

    if tokio::process::Command::new("systemctl")
        .args(["restart", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Io(e.to_string()))?
        .success()
        .not()
    {
        return Err(test_rpc::Error::ServiceStart(
            "Daemon service could not be restarted".to_owned(),
        ));
    };

    wait_for_service_state(ServiceState::Running).await?;
    Ok(())
}

pub async fn get_daemon_environment() -> Result<HashMap<String, String>, test_rpc::Error> {
    let text = tokio::fs::read_to_string(SYSTEMD_OVERRIDE_FILE)
        .await
        .map_err(|err| test_rpc::Error::FileSystem(err.to_string()))?;

    let env: HashMap<String, String> = parse_systemd_env_file(&text)
        .map(|EnvVar { var, value }| (var, value))
        .collect();
    Ok(env)
}

/// Parse a systemd env-file. `input` is assumed to be the entire text content of a systemd-env
/// file.
///
/// Example systemd-env file:
/// ```
/// [Service]
/// Environment="VAR1=pGNqduRFkB4K9C2vijOmUDa2kPtUhArN"
/// Environment="VAR2=JP8YLOc2bsNlrGuD6LVTq7L36obpjzxd"
/// ```
fn parse_systemd_env_file(input: &str) -> impl Iterator<Item = EnvVar> + '_ {
    input
        .lines()
        .map(EnvVar::from_systemd_string)
        .filter_map(|env_var| env_var.ok())
        .inspect(|env_var| log::trace!("Parsed {env_var:?}"))
}

enum ServiceState {
    Running,
    Inactive,
}

async fn wait_for_service_state(awaited_state: ServiceState) -> Result<(), test_rpc::Error> {
    const RETRY_ATTEMPTS: usize = 10;
    let mut attempt = 0;
    loop {
        attempt += 1;
        if attempt > RETRY_ATTEMPTS {
            return Err(test_rpc::Error::ServiceStart(String::from(
                "Awaiting new service state timed out",
            )));
        }

        let output = tokio::process::Command::new("systemctl")
            .args(["status", "mullvad-daemon"])
            .output()
            .await
            .map_err(|e| test_rpc::Error::ServiceNotFound(e.to_string()))?
            .stdout;
        let output = String::from_utf8_lossy(&output);

        match awaited_state {
            ServiceState::Running => {
                if output.contains("active (running)") {
                    break;
                }
            }
            ServiceState::Inactive => {
                if output.contains("inactive (dead)") {
                    break;
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    Ok(())
}

pub fn get_os_version() -> Result<OsVersion, test_rpc::Error> {
    Ok(OsVersion::Linux)
}

#[cfg(test)]
mod test {

    #[test]
    fn parse_systemd_environment_variables() {
        use super::parse_systemd_env_file;
        // Define an example systemd environment file
        let systemd_file = "
        [Service]
        Environment=\"var1=value1\"
        Environment=\"var2=value2\"
        ";

        // Parse the "file"
        let env_vars: Vec<_> = parse_systemd_env_file(systemd_file).collect();

        // Assert that the environment variables it defines are parsed as expected.
        assert_eq!(env_vars.len(), 2);
        let first = env_vars.first().unwrap();
        assert_eq!(first.var, "var1");
        assert_eq!(first.value, "value1");
        let second = env_vars.get(1).unwrap();
        assert_eq!(second.var, "var2");
        assert_eq!(second.value, "value2");
    }
}
