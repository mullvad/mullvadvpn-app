//! Test manager configuration.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to read config")]
    Read(io::Error),
    #[error(display = "Failed to parse config")]
    InvalidConfig(serde_json::Error),
    #[error(display = "Failed to write config")]
    Write(io::Error),
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    pub mullvad_host: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct RuntimeOptions {
    pub display: Display,
    pub keep_changes: bool,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub enum Display {
    #[default]
    None,
    Local,
    Vnc,
}

impl Config {
    async fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::load(path).await.or_else(|error| match error {
            Error::Read(ref io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                Ok(Self::default())
            }
            error => Err(error),
        })
    }

    async fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = tokio::fs::read(path).await.map_err(Error::Read)?;
        serde_json::from_slice(&data).map_err(Error::InvalidConfig)
    }

    async fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let data = serde_json::to_vec_pretty(self).unwrap();
        tokio::fs::write(path, &data).await.map_err(Error::Write)
    }

    pub fn get_vm(&self, name: &str) -> Option<&VmConfig> {
        self.vms.get(name)
    }
}

pub struct ConfigFile {
    path: PathBuf,
    config: Config,
}

impl ConfigFile {
    /// Make config changes and save them to disk
    pub async fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: Config::load_or_default(path).await?,
        })
    }

    /// Make config changes and save them to disk
    pub async fn edit(&mut self, edit: impl FnOnce(&mut Config)) -> Result<(), Error> {
        edit(&mut self.config);
        self.config.save(&self.path).await
    }
}

impl Deref for ConfigFile {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

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
    #[arg(long, required_if_eq("os_type", "linux"))]
    pub architecture: Option<Architecture>,

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
}

impl VmConfig {
    /// Combine authentication details, if all are present
    pub fn get_ssh_options(&self) -> Option<(&str, &str)> {
        Some((self.ssh_user.as_ref()?, self.ssh_password.as_ref()?))
    }

    pub fn get_runner_dir(&self) -> &Path {
        match self.architecture {
            None | Some(Architecture::X64) => self.get_x64_runner_dir(),
            Some(Architecture::Aarch64) => self.get_aarch64_runner_dir(),
        }
    }

    fn get_x64_runner_dir(&self) -> &Path {
        pub const X64_LINUX_TARGET_DIR: &str = "./target/x86_64-unknown-linux-gnu/release";
        pub const X64_WINDOWS_TARGET_DIR: &str = "./target/x86_64-pc-windows-gnu/release";
        pub const X64_MACOS_TARGET_DIR: &str = "./target/x86_64-apple-darwin/release";

        match self.os_type {
            OsType::Linux => Path::new(X64_LINUX_TARGET_DIR),
            OsType::Windows => Path::new(X64_WINDOWS_TARGET_DIR),
            OsType::Macos => Path::new(X64_MACOS_TARGET_DIR),
        }
    }

    fn get_aarch64_runner_dir(&self) -> &Path {
        pub const AARCH64_LINUX_TARGET_DIR: &str = "./target/aarch64-unknown-linux-gnu/release";
        pub const AARCH64_MACOS_TARGET_DIR: &str = "./target/aarch64-apple-darwin/release";

        match self.os_type {
            OsType::Linux => Path::new(AARCH64_LINUX_TARGET_DIR),
            OsType::Macos => Path::new(AARCH64_MACOS_TARGET_DIR),
            _ => unimplemented!(),
        }
    }
}

#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VmType {
    /// QEMU VM
    Qemu,
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
    pub fn get_identifiers(&self) -> &[&'static str] {
        match self {
            Architecture::X64 => &["x86_64", "amd64"],
            Architecture::Aarch64 => &["arm64", "aarch64"],
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
