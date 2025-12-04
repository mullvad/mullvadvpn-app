use anyhow::{Context as _, anyhow};
use libc::pid_t;
use nix::{errno::Errno, unistd::Pid};
use std::{
    env,
    ffi::CStr,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
};
use talpid_types::{
    ErrorExt,
    cgroup::{SPLIT_TUNNEL_CGROUP_NAME, find_net_cls_mount},
};

pub const DEFAULT_NET_CLS_DIR: &str = "/sys/fs/cgroup/net_cls";

// TODO: respect this?
pub const NET_CLS_DIR_OVERRIDE_ENV_VAR: &str = "TALPID_NET_CLS_MOUNT_DIR";

/// Identifies packets coming from the cgroup.
/// This should be an arbitrary but unique integer.
pub const NET_CLS_CLASSID: u32 = 0x4d9f41;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Unable to create cgroup.
    #[error("Unable to initialize net_cls cgroup instance")]
    InitNetClsCGroup(#[source] nix::Error),

    /// Unable to create cgroup.
    #[error("Unable to create cgroup for excluded processes")]
    CreateCGroup(#[source] io::Error),

    /// Split tunneling is unavailable
    #[error("Failed to set up split tunneling")]
    Unavailable,

    /// Unable to set class ID for cgroup.
    #[error("Unable to set cgroup class ID")]
    SetCGroupClassId(#[source] io::Error),

    /// Unable to add PID to cgroup.procs.
    #[error("Unable to add PID to cgroup.procs")]
    AddCGroupPid(#[source] io::Error),

    /// Unable to remove PID to cgroup.procs.
    #[error("Unable to remove PID from cgroup")]
    RemoveCGroupPid(#[source] io::Error),

    /// Unable to read cgroup.procs.
    #[error("Unable to obtain PIDs from cgroup.procs")]
    ListCGroupPids(#[source] io::Error),

    /// Unable to read /proc/mounts
    #[error("Failed to read /proc/mounts")]
    ListMounts(#[source] io::Error),
}

/// Set up a v1 cgroup used to track PIDs for split tunneling.
pub fn create_cgroup_v1() -> Result<PathBuf, Error> {
    if let Some(net_cls_path) = find_net_cls_mount().map_err(Error::ListMounts)? {
        return Ok(net_cls_path);
    }

    let net_cls_dir = env::var(NET_CLS_DIR_OVERRIDE_ENV_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_NET_CLS_DIR));

    if !net_cls_dir.exists() {
        fs::create_dir_all(&net_cls_dir).map_err(Error::CreateCGroup)?;
    }

    // https://www.kernel.org/doc/Documentation/cgroup-v1/net_cls.txt
    nix::mount::mount(
        Some("net_cls"),
        &net_cls_dir,
        Some("cgroup"),
        nix::mount::MsFlags::empty(),
        Some("net_cls"),
    )
    .map_err(Error::InitNetClsCGroup)?;

    Ok(net_cls_dir)
}

pub fn setup_exclusion_group(net_cls_path: &Path) -> Result<PathBuf, Error> {
    let exclusions_dir = net_cls_path.join(SPLIT_TUNNEL_CGROUP_NAME);
    if !exclusions_dir.exists() {
        fs::create_dir(exclusions_dir.clone()).map_err(Error::CreateCGroup)?;
    }

    // Assign our unique id to the net_cls
    let classid_path = exclusions_dir.join("net_cls.classid");
    fs::write(classid_path, NET_CLS_CLASSID.to_string().as_bytes())
        .map_err(Error::SetCGroupClassId)?;

    Ok(exclusions_dir)
}

/// A handle to a v1 cgroup
pub struct CGroup1 {
    /// Absolute path of the cgroup, e.g. `/sys/fs/cgroup/net_cls/foobar`
    path: PathBuf,

    /// `cgroup.procs` is used to add and list PIDs in the cgroup2.
    procs: File,
}

impl CGroup1 {
    /// Open the cgroup2 at `path`.
    ///
    /// `path` must be a directory in the `net_cls` filesystem.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, super::Error> {
        let path = path.into();

        let procs_path = path.join("cgroup.procs");
        let procs = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(false)
            .open(&procs_path)
            .with_context(|| anyhow!("Failed to open {procs_path:?}"))?;

        Ok(CGroup1 { path, procs })
    }

    /// Create or open a child to the current cgroup called `name`.
    ///
    /// If the child alread exists, open it.
    pub fn create_or_open_child(&self, name: &str) -> Result<Self, super::Error> {
        let child_path = self.path.join(name);
        match nix::unistd::mkdir(&child_path, nix::sys::stat::Mode::from_bits_truncate(0o755)) {
            Ok(_) => log::debug!("cgroup1 {name:?} created"),
            Err(Errno::EEXIST) => log::debug!("cgroup1 already exists"),
            Err(e) => Err(e).context("Failed to create cgroup1")?,
        }

        Self::open(child_path)
    }

    /// Try to clone the cgroup2 handle.
    ///
    /// This is fallible because cloning file descriptors can fail.
    pub fn try_clone(&self) -> Result<Self, super::Error> {
        Ok(Self {
            path: self.path.clone(),
            procs: self
                .procs
                .try_clone()
                .context("Failed to clone procs file handle")?,
        })
    }

    /// Assign a process to this cgroup2.
    pub fn add_pid(&self, pid: Pid) -> Result<(), super::Error> {
        // Format the PID as a string
        let mut pid_buf = [0u8; 16];
        write!(&mut pid_buf[..], "{pid}").expect("buf is large enough");
        let pid_str = CStr::from_bytes_until_nul(&pid_buf).expect("buf contains null");

        // Write PID to `cgroup.procs`.
        nix::unistd::write(&self.procs, pid_str.to_bytes())
            .with_context(|| anyhow!("Failed to add process {pid} to cgroup2"))?;

        Ok(())
    }

    /// List all PIDs in this cgroup2.
    pub fn list_pids(&mut self) -> Result<Vec<pid_t>, super::Error> {
        let mut file = &self.procs;
        let mut pids = String::new();

        file.seek(io::SeekFrom::Start(0))
            .and_then(|_| file.read_to_string(&mut pids))
            .with_context(|| anyhow!("Failed to read pids from {:?}", self.path))?;

        let pids = pids
            .lines()
            .map(|line| line.trim())
            .filter_map(|line| {
                line.parse::<pid_t>()
                    .inspect_err(|e| log::trace!("Failed to parse PID {line:?}: {e}"))
                    .ok()
            })
            .collect();
        Ok(pids)
    }

    pub fn set_net_cls_id(&self, net_cls_classid: u32) -> Result<(), super::Error> {
        // Assign our unique id to the net_cls
        let classid_path = self.path.join("net_cls.classid");
        fs::write(&classid_path, net_cls_classid.to_string().as_bytes())
            .with_context(|| anyhow!("Failed to write NET_CLS_CLASSID to {classid_path:?}"))?;
        Ok(())
    }
}
