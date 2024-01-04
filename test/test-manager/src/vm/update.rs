use crate::config::{OsType, PackageType, Provisioner, VmConfig};
use crate::vm::ssh::SSHSession;
use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug)]
pub enum Update {
    Logs(Vec<String>),
    Nothing,
}

/// Update system packages in a VM.
///
/// Note that this function is blocking.
pub fn packages(config: &VmConfig, guest_ip: std::net::IpAddr) -> Result<Update> {
    match config.provisioner {
        Provisioner::Noop => return Ok(Update::Nothing),
        Provisioner::Ssh => (),
    }
    // User SSH session to execute package manager update command.
    // This will of course be dependant on the target platform.
    let commands = match (config.os_type, config.package_type) {
        (OsType::Linux, Some(PackageType::Deb)) => {
            Some(vec!["sudo apt-get update", "sudo apt-get upgrade"])
        }
        (OsType::Linux, Some(PackageType::Rpm)) => Some(vec!["sudo dnf update"]),
        (OsType::Linux, _) => None,
        (OsType::Macos | OsType::Windows, _) => None,
    };

    // Issue the update command(s).
    let result = match commands {
        None => {
            log::info!("No update command was found");
            log::debug!(
                "Tried to invoke package update for platform {:?} with package type {:?}",
                config.os_type,
                config.package_type
            );
            Update::Nothing
        }
        Some(commands) => {
            log::info!("retrieving SSH credentials");
            let (username, password) = config.get_ssh_options().context("missing SSH config")?;
            let ssh = SSHSession::connect(username.to_string(), password.to_string(), guest_ip)?;
            let output: Result<Vec<_>> = commands
                .iter()
                .map(|command| {
                    log::info!("Running {command} in guest");
                    ssh.exec_blocking(command)
                })
                .collect();
            Update::Logs(output?)
        }
    };

    Ok(result)
}

// Pretty-printing for an `Update` action.
impl fmt::Display for Update {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Update::Nothing => write!(formatter, "Nothing was updated"),
            Update::Logs(output) => output
                .iter()
                .try_for_each(|output| formatter.write_str(output)),
        }
    }
}
