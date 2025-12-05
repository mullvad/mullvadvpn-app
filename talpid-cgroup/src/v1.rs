use crate::find_net_cls_mount;
use anyhow::{Context as _, anyhow};
use nix::{errno::Errno, libc::pid_t, unistd::Pid};
use std::{
    env,
    ffi::CStr,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::PathBuf,
};

/// Path where cgroup1 will be mounted.
pub const NET_CLS_DIR_OVERRIDE_ENV_VAR: &str = "TALPID_NET_CLS_MOUNT_DIR";

/// The path where linux normally mounts the net_cls cgroup v1 filesystem.
pub const DEFAULT_NET_CLS_DIR: &str = "/sys/fs/cgroup/net_cls";

/// Identifies packets coming from the cgroup.
/// This should be an arbitrary but unique integer.
pub const NET_CLS_CLASSID: u32 = 0x4d9f41;

/// A handle to a v1 net_cls cgroup
pub struct CGroup1 {
    /// Absolute path of the cgroup, e.g. `/sys/fs/cgroup/net_cls/foobar`
    path: PathBuf,

    /// `cgroup.procs` is used to add and list PIDs in the cgroup2.
    procs: File,
}

impl CGroup1 {
    /// Open the root net_cls cgroup at [`NET_CLS_DIR_OVERRIDE_ENV_VAR`] (or [`DEFAULT_NET_CLS_DIR`] if env variable is unset), creating if if it doesn't exist.
    pub fn open_root() -> Result<Self, super::Error> {
        if let Some(net_cls_path) = find_net_cls_mount()? {
            return Self::open(net_cls_path);
        }

        // mkdir and mount the net_cls dir if it doesn't exist
        // https://www.kernel.org/doc/Documentation/cgroup-v1/net_cls.txt
        let net_cls_dir = env::var(NET_CLS_DIR_OVERRIDE_ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_NET_CLS_DIR));
        if !net_cls_dir.exists() {
            fs::create_dir(&net_cls_dir).with_context(|| {
                anyhow!("Unable to create cgroup {net_cls_dir:?} for excluded processes")
            })?;
        }
        nix::mount::mount(
            Some("net_cls"),
            &net_cls_dir,
            Some("cgroup"),
            nix::mount::MsFlags::empty(),
            Some("net_cls"),
        )
        .with_context(|| anyhow!("Unable to mount net_cls cgroup instance {net_cls_dir:?}"))?;
        // then open it
        Self::open(net_cls_dir)
    }

    /// Open the [`CGroup1`] at `path`.
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

    /// Set the classid for this net_cls cgroup.
    pub fn set_net_cls_id(&self, net_cls_classid: u32) -> Result<(), super::Error> {
        // Assign our unique id to the net_cls
        let classid_path = self.path.join("net_cls.classid");
        fs::write(&classid_path, net_cls_classid.to_string().as_bytes())
            .with_context(|| anyhow!("Failed to write NET_CLS_CLASSID to {classid_path:?}"))?;
        Ok(())
    }
}
