//! Virtual machine configuration.

use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(clap::Args, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct VmConfig {
    /// Type of virtual machine to use
    pub vm_type: VmType,

    /// Path to a VM disk image
    pub image_path: String,

    /// Type of operating system.
    pub os_type: OsType,

    /// Package type to use, e.g. deb or rpm
    #[arg(long, required_if_eq("os_type", "linux"))]
    pub package_type: Option<PackageType>,

    /// CPU architecture
    #[arg(long)]
    pub architecture: Architecture,

    /// Tool to use for provisioning
    #[arg(long, default_value = "noop")]
    pub provisioner: Provisioner,

    /// Username to use for SSH
    #[arg(long, required_if_eq("provisioner", "ssh"))]
    pub ssh_user: Option<String>,

    /// Password to use for SSH
    #[arg(long, required_if_eq("provisioner", "ssh"))]
    pub ssh_password: Option<String>,

    /// Additional disk images to mount/include
    #[arg(long)]
    pub disks: Vec<String>,

    /// Where artifacts, such as app packages, are stored.
    /// Usually /opt/testing on Linux.
    #[arg(long)]
    pub artifacts_dir: Option<String>,

    /// Emulate a TPM. This also enables UEFI implicitly
    #[serde(default)]
    #[arg(long)]
    pub tpm: bool,

    /// Override the path to `OVMF_VARS.secboot.fd`. Requires `tpm`.
    #[serde(default)]
    #[arg(long, requires("tpm"))]
    pub ovmf_vars_path: Option<String>,

    /// Override the path to `OVMF_CODE.secboot.fd`. Requires `tpm`.
    #[serde(default)]
    #[arg(long, requires("tpm"))]
    pub ovmf_code_path: Option<String>,

    /// Number of vCPUs
    #[serde(default)]
    #[arg(long)]
    pub vcpus: Option<usize>,

    /// Amount of memory, in MBs
    #[serde(default)]
    #[arg(long)]
    pub memory: Option<usize>,
}

impl VmConfig {
    /// Combine authentication details, if all are present
    pub fn get_ssh_options(&self) -> Option<(&str, &str)> {
        Some((self.ssh_user.as_ref()?, self.ssh_password.as_ref()?))
    }

    pub fn get_default_runner_dir(&self) -> PathBuf {
        let target_dir = self.get_target_dir();
        let subdir = match self.architecture {
            Architecture::X64 => self.get_x64_runner_subdir(),
            Architecture::Aarch64 => self.get_aarch64_runner_subdir(),
        };

        target_dir.join(subdir)
    }

    fn get_x64_runner_subdir(&self) -> &Path {
        pub const X64_LINUX_TARGET_DIR: &str = "x86_64-unknown-linux-gnu/release";
        pub const X64_WINDOWS_TARGET_DIR: &str = "x86_64-pc-windows-gnu/release";
        pub const X64_MACOS_TARGET_DIR: &str = "x86_64-apple-darwin/release";

        match self.os_type {
            OsType::Linux => Path::new(X64_LINUX_TARGET_DIR),
            OsType::Windows => Path::new(X64_WINDOWS_TARGET_DIR),
            OsType::Macos => Path::new(X64_MACOS_TARGET_DIR),
        }
    }

    fn get_aarch64_runner_subdir(&self) -> &Path {
        pub const AARCH64_LINUX_TARGET_DIR: &str = "aarch64-unknown-linux-gnu/release";
        pub const AARCH64_MACOS_TARGET_DIR: &str = "aarch64-apple-darwin/release";

        match self.os_type {
            OsType::Linux => Path::new(AARCH64_LINUX_TARGET_DIR),
            OsType::Macos => Path::new(AARCH64_MACOS_TARGET_DIR),
            _ => unimplemented!(),
        }
    }

    fn get_target_dir(&self) -> PathBuf {
        env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| "./target".into())
            .into()
    }
}

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VmType {
    #[cfg(target_os = "linux")]
    /// QEMU VM
    Qemu,
    #[cfg(target_os = "macos")]
    /// Tart VM
    Tart,
}

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OsType {
    Windows,
    Linux,
    Macos,
}

impl From<OsType> for test_rpc::meta::Os {
    fn from(ostype: OsType) -> Self {
        match ostype {
            OsType::Windows => Self::Windows,
            OsType::Linux => Self::Linux,
            OsType::Macos => Self::Macos,
        }
    }
}

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    Deb,
    Rpm,
}

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    X64,
    Aarch64,
}

impl Architecture {
    pub fn get_identifiers(self) -> Vec<&'static str> {
        match self {
            Architecture::X64 => vec!["x86_64", "amd64"],
            Architecture::Aarch64 => vec!["arm64", "aarch64"],
        }
    }
}

#[derive(clap::ValueEnum, Default, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Provisioner {
    /// Do nothing: The image already includes a test runner service
    #[default]
    Noop,
    /// Set up test runner over SSH.
    Ssh,
}
