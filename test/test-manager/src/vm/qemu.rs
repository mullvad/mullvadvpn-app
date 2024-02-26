use crate::{
    config::{self, Config, VmConfig},
    vm::{logging::forward_logs, util::find_pty},
};
use async_tempfile::TempFile;
use regex::Regex;
use std::{
    io,
    net::IpAddr,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};
use tokio::{
    fs,
    process::{Child, Command},
    time::timeout,
};
use uuid::Uuid;

use super::{network, VmInstance};

const LOG_PREFIX: &str = "[qemu] ";
const STDERR_LOG_LEVEL: log::Level = log::Level::Error;
const STDOUT_LOG_LEVEL: log::Level = log::Level::Debug;
const OBTAIN_IP_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to set up network")]
    Network(network::linux::Error),
    #[error("Failed to start QEMU")]
    StartQemu(io::Error),
    #[error("QEMU exited unexpectedly")]
    QemuFailed(Option<ExitStatus>),
    #[error("Could not find pty")]
    NoPty,
    #[error("Could not find IP address of guest")]
    NoIpAddr,
    #[error("Failed to copy OVMF vars")]
    CopyOvmfVars(io::Error),
    #[error("Failed to wrap OVMF vars copy in tempfile object")]
    WrapOvmfVars,
    #[error("Failed to start swtpm")]
    StartTpmEmulator(io::Error),
    #[error("swtpm failed")]
    TpmEmulator(io::Error),
    #[error("Timed out waiting for swtpm socket")]
    TpmSocketTimeout,
    #[error("Failed to create temp dir")]
    MkTempDir(io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct QemuInstance {
    pub pty_path: String,
    pub ip_addr: IpAddr,
    child: Child,
    _network_handle: network::linux::NetworkHandle,
    _ovmf_handle: Option<OvmfHandle>,
    _tpm_emulator: Option<TpmEmulator>,
}

#[async_trait::async_trait]
impl VmInstance for QemuInstance {
    fn get_pty(&self) -> &str {
        &self.pty_path
    }

    fn get_ip(&self) -> &IpAddr {
        &self.ip_addr
    }

    async fn wait(&mut self) {
        let _ = self.child.wait().await;
    }
}

pub async fn run(config: &Config, vm_config: &VmConfig) -> Result<QemuInstance> {
    let mut network_handle = network::linux::setup_test_network()
        .await
        .map_err(Error::Network)?;

    let mut qemu_cmd = Command::new("qemu-system-x86_64");
    qemu_cmd.args([
        "-cpu",
        "host",
        "-accel",
        "kvm",
        "-m",
        "4096",
        "-smp",
        "2",
        "-drive",
        &format!("file={}", vm_config.image_path),
        "-device",
        "virtio-serial-pci",
        "-serial",
        "pty",
        // attach to TAP interface
        "-nic",
        &format!(
            "tap,ifname={},script=no,downscript=no",
            network::linux::TAP_NAME
        ),
        "-device",
        "nec-usb-xhci,id=xhci",
    ]);

    if !config.runtime_opts.keep_changes {
        qemu_cmd.arg("-snapshot");
    }

    match config.runtime_opts.display {
        config::Display::None => {
            qemu_cmd.args(["-display", "none"]);
        }
        config::Display::Local => (),
        config::Display::Vnc => {
            log::debug!("Running VNC server on :1");
            qemu_cmd.args(["-display", "vnc=:1"]);
        }
    }

    for (i, disk) in vm_config.disks.iter().enumerate() {
        qemu_cmd.args([
            "-drive",
            &format!("if=none,id=disk{i},file={disk}"),
            "-device",
            &format!("usb-storage,drive=disk{i},bus=xhci.0"),
        ]);
    }

    // Configure OVMF. Currently, this is enabled implicitly if using a TPM
    let ovmf_handle = if vm_config.tpm {
        let handle = OvmfHandle::new().await?;
        handle.append_qemu_args(&mut qemu_cmd);
        Some(handle)
    } else {
        None
    };

    // Run software TPM emulator
    let tpm_emulator = if vm_config.tpm {
        let handle = TpmEmulator::run().await?;
        handle.append_qemu_args(&mut qemu_cmd);
        Some(handle)
    } else {
        None
    };

    qemu_cmd.stdin(Stdio::piped());
    qemu_cmd.stdout(Stdio::piped());
    qemu_cmd.stderr(Stdio::piped());

    qemu_cmd.kill_on_drop(true);

    let mut child = qemu_cmd.spawn().map_err(Error::StartQemu)?;

    tokio::spawn(forward_logs(
        LOG_PREFIX,
        child.stderr.take().unwrap(),
        STDERR_LOG_LEVEL,
    ));

    // find pty in stdout
    // match: char device redirected to /dev/pts/0 (label serial0)
    let re = Regex::new(r"char device redirected to ([/a-zA-Z0-9]+) \(").unwrap();
    let pty_path = find_pty(re, &mut child, STDOUT_LOG_LEVEL, LOG_PREFIX)
        .await
        .map_err(|_error| {
            if let Ok(status) = child.try_wait() {
                return Error::QemuFailed(status);
            }
            Error::NoPty
        })?;

    tokio::spawn(forward_logs(
        LOG_PREFIX,
        child.stdout.take().unwrap(),
        STDOUT_LOG_LEVEL,
    ));

    log::debug!("Waiting for IP address");
    let ip_addr = timeout(OBTAIN_IP_TIMEOUT, network_handle.first_dhcp_ack())
        .await
        .map_err(|_| Error::NoIpAddr)?
        .ok_or(Error::NoIpAddr)?;
    log::debug!("Guest IP: {ip_addr}");

    Ok(QemuInstance {
        pty_path,
        ip_addr,
        child,
        _network_handle: network_handle,
        _ovmf_handle: ovmf_handle,
        _tpm_emulator: tpm_emulator,
    })
}

/// Used to set up UEFI and append options to the QEMU command
struct OvmfHandle {
    temp_vars: TempFile,
}

impl OvmfHandle {
    pub async fn new() -> Result<Self> {
        const OVMF_VARS_PATH: &str = "/usr/share/OVMF/OVMF_VARS.secboot.fd";

        // Create a local copy of OVMF_VARS
        let temp_vars_path = random_tempfile_name();
        fs::copy(OVMF_VARS_PATH, &temp_vars_path)
            .await
            .map_err(Error::CopyOvmfVars)?;

        let temp_vars = TempFile::from_existing(temp_vars_path, async_tempfile::Ownership::Owned)
            .await
            .map_err(|_| Error::WrapOvmfVars)?;
        Ok(OvmfHandle { temp_vars })
    }

    pub fn append_qemu_args(&self, qemu_cmd: &mut Command) {
        const OVMF_CODE_PATH: &str = "/usr/share/OVMF/OVMF_CODE.secboot.fd";

        qemu_cmd.args([
            "-global",
            "driver=cfi.pflash01,property=secure,value=on",
            "-drive",
            &format!("if=pflash,format=raw,unit=0,file={OVMF_CODE_PATH},readonly=on"),
            "-drive",
            &format!(
                "if=pflash,format=raw,unit=1,file={}",
                self.temp_vars.file_path().display()
            ),
            // Q35 supports secure boot
            "-machine",
            "q35,smm=on",
        ]);
    }
}

/// Runs a TPM emulator
struct TpmEmulator {
    handle: tokio::task::JoinHandle<Result<()>>,
    sock_path: PathBuf,
}

impl TpmEmulator {
    pub async fn run() -> Result<Self> {
        let temp_dir = TempDir::new().await?;
        let mut cmd = Command::new("swtpm");

        let sock_path = temp_dir.0.join("tpmsock");

        cmd.args([
            "socket",
            "-t",
            "--ctrl",
            &format!("type=unixio,path={}", sock_path.display()),
            "--tpmstate",
            &format!("dir={}", temp_dir.0.display()),
            "--tpm2",
        ]);

        cmd.kill_on_drop(true);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Start swtpm
        let mut child = cmd.spawn().map_err(Error::StartTpmEmulator)?;

        tokio::spawn(forward_logs(
            "[swtpm] ",
            child.stdout.take().unwrap(),
            STDOUT_LOG_LEVEL,
        ));
        tokio::spawn(forward_logs(
            "[swtpm] ",
            child.stderr.take().unwrap(),
            STDERR_LOG_LEVEL,
        ));

        let handle = tokio::spawn(async move {
            let output = child.wait().await.map_err(Error::TpmEmulator)?;

            if !output.success() {
                log::error!("swtpm failed: {}", output);
            }

            temp_dir.delete().await;

            Ok(())
        });

        const SOCKET_TIMEOUT: Duration = Duration::from_secs(10);

        // Wait for socket to be created
        timeout(SOCKET_TIMEOUT, async {
            if sock_path.exists() {
                return;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        })
        .await
        .map_err(|_| {
            handle.abort();
            Error::TpmSocketTimeout
        })?;

        Ok(Self { handle, sock_path })
    }

    pub fn append_qemu_args(&self, qemu_cmd: &mut Command) {
        qemu_cmd.args([
            "-tpmdev",
            "emulator,id=tpm0,chardev=chrtpm",
            "-chardev",
            &format!("socket,id=chrtpm,path={}", self.sock_path.display()),
            "-device",
            "tpm-tis,tpmdev=tpm0",
        ]);
    }
}

impl Drop for TpmEmulator {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct TempDir(PathBuf);

impl TempDir {
    pub async fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join(Uuid::new_v4().to_string());
        tokio::fs::create_dir_all(&temp_dir)
            .await
            .map_err(Error::MkTempDir)?;
        Ok(Self(temp_dir))
    }

    pub async fn delete(self) {
        let _ = fs::remove_dir_all(&self.0).await;
        std::mem::forget(self);
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn random_tempfile_name() -> PathBuf {
    std::env::temp_dir().join(format!("tmp{}", Uuid::new_v4()))
}
