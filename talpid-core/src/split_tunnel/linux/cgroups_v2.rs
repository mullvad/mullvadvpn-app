use anyhow::{Context, anyhow};
use libc::pid_t;
use nix::{errno::Errno, unistd::Pid};
use std::{
    ffi::CStr,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};

use super::Error;

/// A handle to a cgroup2
pub struct CGroup2 {
    /// Absolute path of the cgroup2, e.g. `/sys/fs/cgroup/foobar`
    path: PathBuf,

    /// inode of the cgroup2 directory
    inode: u64,

    /// `cgroup.procs` is used to add and list PIDs in the cgroup2.
    procs: File,
}

impl CGroup2 {
    /// Open the cgroup2 at `path`.
    ///
    /// `path` must be a directory in the `cgroup2` filesystem.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();

        let procs_path = path.join("cgroup.procs");
        let procs = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(false)
            .open(&procs_path)
            .with_context(|| anyhow!("Failed to open {procs_path:?}"))?;

        let meta = fs::metadata(&path).with_context(|| anyhow!("Failed to stat {path:?}"))?;

        Ok(CGroup2 {
            path,
            inode: meta.ino(),
            procs,
        })
    }

    /// Create or open a child to the current cgroup2 called `name`.
    ///
    /// If the child alread exists, open it.
    pub fn create_or_open_child(&self, name: &str) -> Result<Self, Error> {
        let child_path = self.path.join(name);
        match nix::unistd::mkdir(&child_path, nix::sys::stat::Mode::from_bits_truncate(0o755)) {
            Ok(_) => log::debug!("cgroup2 {name:?} created"),
            Err(Errno::EEXIST) => log::debug!("cgroup2 already exists"),
            Err(e) => Err(e).context("Failed to create cgroup2")?,
        }

        Self::open(child_path)
    }

    /// Try to clone the cgroup2 handle.
    ///
    /// This is fallible because cloning file descriptors can fail.
    pub fn try_clone(&self) -> Result<Self, Error> {
        Ok(Self {
            path: self.path.clone(),
            inode: self.inode,
            procs: self
                .procs
                .try_clone()
                .context("Failed to clone procs file handle")?,
        })
    }

    /// Assign a process to this cgroup2.
    pub fn add_pid(&self, pid: Pid) -> Result<(), Error> {
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
    pub fn list_pids(&mut self) -> Result<Vec<pid_t>, Error> {
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

    /// Get the inode of the cgroup2
    pub const fn inode(&self) -> u64 {
        self.inode
    }
}
