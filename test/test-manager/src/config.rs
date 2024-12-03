//! Test manager configuration.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, io,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::tests::config::DEFAULT_MULLVAD_HOST;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not find config dir")]
    FindConfigDir,
    #[error("Could not create config dir")]
    CreateConfigDir(#[source] io::Error),
    #[error("Failed to read config")]
    Read(#[source] io::Error),
    #[error("Failed to parse config")]
    InvalidConfig(#[from] serde_json::Error),
    #[error("Failed to write config")]
    Write(#[source] io::Error),
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

    /// Get the Mullvad host to use.
    ///
    /// Defaults to [`DEFAULT_MULLVAD_HOST`] if the host was not provided in the [`ConfigFile`].
    pub fn get_host(&self) -> String {
        self.mullvad_host.clone().unwrap_or_else(|| {
            log::debug!("No Mullvad host has been set explicitly. Falling back to default host");
            DEFAULT_MULLVAD_HOST.to_owned()
        })
    }
}

pub struct ConfigFile {
    path: PathBuf,
    config: Config,
}

impl ConfigFile {
    /// Make config changes and save them to disk
    pub async fn load_or_default() -> Result<Self, Error> {
        Self::load_or_default_inner(Self::get_config_path()?).await
    }

    /// Get configuration file path
    fn get_config_path() -> Result<PathBuf, Error> {
        Ok(Self::get_config_dir()?.join("config.json"))
    }

    /// Get configuration file directory
    fn get_config_dir() -> Result<PathBuf, Error> {
        let dir = dirs::config_dir()
            .ok_or(Error::FindConfigDir)?
            .join("mullvad-test");
        Ok(dir)
    }

    /// Create configuration file directory if it does not exist
    async fn ensure_config_dir() -> Result<(), Error> {
        tokio::fs::create_dir_all(Self::get_config_dir()?)
            .await
            .map_err(Error::CreateConfigDir)
    }

    /// Make config changes and save them to disk
    async fn load_or_default_inner<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: Config::load_or_default(path).await?,
        })
    }

    /// Make config changes and save them to disk
    pub async fn edit(&mut self, edit: impl FnOnce(&mut Config)) -> Result<(), Error> {
        Self::ensure_config_dir().await?;

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
    ///
    /// TODO: Remove default x86_64, do not assume the system we're virtualizing
    #[arg(long)]
    #[serde(default = "Architecture::host_arch")]
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

    /// Figure out the architecture of the host test-manager was compiled for
    pub const fn host_arch() -> Architecture {
        // Panic at compile time
        const ARCH: Architecture = if cfg!(target_arch = "x86_64") {
            Architecture::X64
        } else if cfg!(target_arch = "aarch64") {
            Architecture::Aarch64
        } else {
            panic!("Unsupported target arch")
        };
        ARCH
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
