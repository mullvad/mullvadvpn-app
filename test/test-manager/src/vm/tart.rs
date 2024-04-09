use crate::config::{self, Config, VmConfig};
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::{net::IpAddr, process::Stdio, time::Duration};
use tokio::process::{Child, Command};
use uuid::Uuid;

use super::{logging::forward_logs, util::find_pty, VmInstance};

const LOG_PREFIX: &str = "[tart] ";
const STDERR_LOG_LEVEL: log::Level = log::Level::Error;
const STDOUT_LOG_LEVEL: log::Level = log::Level::Debug;
const OBTAIN_IP_TIMEOUT: Duration = Duration::from_secs(60);

pub struct TartInstance {
    pub pty_path: String,
    pub ip_addr: IpAddr,
    child: Child,
    machine_copy: Option<MachineCopy>,
}

#[async_trait::async_trait]
impl VmInstance for TartInstance {
    fn get_pty(&self) -> &str {
        &self.pty_path
    }

    fn get_ip(&self) -> &IpAddr {
        &self.ip_addr
    }

    async fn wait(&mut self) {
        let _ = self.child.wait().await;
        if let Some(machine) = self.machine_copy.take() {
            machine.cleanup().await;
        }
    }
}

pub async fn run(config: &Config, vm_config: &VmConfig) -> Result<TartInstance> {
    super::network::macos::setup_test_network()
        .await
        .context("Failed to set up networking")?;

    // Create a temporary clone of the machine
    let machine_copy = if config.runtime_opts.keep_changes {
        MachineCopy::borrow_vm(&vm_config.image_path)
    } else {
        MachineCopy::clone_vm(&vm_config.image_path).await?
    };

    // Start VM
    let mut tart_cmd = Command::new("tart");
    tart_cmd.args(["run", &machine_copy.name, "--serial"]);

    if !vm_config.disks.is_empty() {
        log::warn!("Mounting disks is not yet supported")
    }

    match config.runtime_opts.display {
        config::Display::None => {
            tart_cmd.arg("--no-graphics");
        }
        config::Display::Local => (),
        config::Display::Vnc => {
            // tart_cmd.args(["--vnc-experimental", "--no-graphics"]);
            tart_cmd.args(["--vnc", "--no-graphics"]);
        }
    }

    tart_cmd.stdin(Stdio::piped());
    tart_cmd.stdout(Stdio::piped());
    tart_cmd.stderr(Stdio::piped());

    tart_cmd.kill_on_drop(true);

    let mut child = tart_cmd.spawn().context("Failed to start Tart")?;

    tokio::spawn(forward_logs(
        LOG_PREFIX,
        child.stderr.take().unwrap(),
        STDERR_LOG_LEVEL,
    ));

    // find pty in stdout
    // match: Successfully open pty /dev/ttys001
    let re = Regex::new(r"Successfully open pty ([/a-zA-Z0-9]+)$").unwrap();
    let pty_path = find_pty(re, &mut child, STDOUT_LOG_LEVEL, LOG_PREFIX)
        .await
        .map_err(|_error| {
            if let Ok(Some(status)) = child.try_wait() {
                return anyhow!("'tart start' failed: {status}");
            }
            anyhow!("Could not find pty")
        })?;

    tokio::spawn(forward_logs(
        LOG_PREFIX,
        child.stdout.take().unwrap(),
        STDOUT_LOG_LEVEL,
    ));

    // Get IP address of VM
    log::debug!("Waiting for IP address");

    let mut tart_cmd = Command::new("tart");
    tart_cmd.args([
        "ip",
        &machine_copy.name,
        "--wait",
        &format!("{}", OBTAIN_IP_TIMEOUT.as_secs()),
    ]);
    let output = tart_cmd.output().await.context("Could not obtain VM IP")?;
    let ip_addr = std::str::from_utf8(&output.stdout)
        .context("'tart ip' returned non-UTF8")?
        .trim()
        .parse()
        .context("Could not parse IP address from 'tart ip'")?;

    log::debug!("Guest IP: {ip_addr}");

    // The tunnel must be configured after the virtual machine is up, or macOS refuses to assign an
    // IP. The reasons for this are poorly understood.
    crate::vm::network::macos::configure_tunnel().await?;

    Ok(TartInstance {
        child,
        pty_path,
        ip_addr,
        machine_copy: Some(machine_copy),
    })
}

/// Handle for a transient or borrowed Tart VM.
/// TODO: Prune VMs we fail to delete them somehow.
pub struct MachineCopy {
    name: String,
    should_destroy: bool,
}

impl MachineCopy {
    /// Use an existing VM and save all changes to it.
    pub fn borrow_vm(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            should_destroy: false,
        }
    }

    /// Clone an existing VM and destroy changes when self is dropped.
    pub async fn clone_vm(name: &str) -> Result<Self> {
        let clone_name = format!("test-{}", Uuid::new_v4());

        let mut tart_cmd = Command::new("tart");
        tart_cmd.args(["clone", name, &clone_name]);
        let output = tart_cmd
            .status()
            .await
            .context("failed to run 'tart clone'")?;
        if !output.success() {
            return Err(anyhow!("'tart clone' failed: {output}"));
        }

        Ok(Self {
            name: clone_name,
            should_destroy: true,
        })
    }

    pub async fn cleanup(mut self) {
        let _ = tokio::task::spawn_blocking(move || self.try_destroy()).await;
    }

    fn try_destroy(&mut self) {
        if !self.should_destroy {
            return;
        }

        if let Err(error) = self.destroy_inner() {
            log::error!("Failed to destroy Tart clone: {error}");
        } else {
            self.should_destroy = false;
        }
    }

    fn destroy_inner(&mut self) -> Result<()> {
        use std::process::Command;

        let mut tart_cmd = Command::new("tart");
        tart_cmd.args(["delete", &self.name]);
        let output = tart_cmd.status().context("Failed to run 'tart delete'")?;
        if !output.success() {
            return Err(anyhow!("'tart delete' failed: {output}"));
        }

        Ok(())
    }
}

impl Drop for MachineCopy {
    fn drop(&mut self) {
        self.try_destroy();
    }
}
