//! Linux split-tunneling implementation using cgroup2.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

use anyhow::{Context, anyhow};
use libc::pid_t;
use nix::{errno::Errno, unistd::Pid};
use std::{
    ffi::CStr,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};
use talpid_types::cgroup::SPLIT_TUNNEL_CGROUP_NAME;

/// The path where we mount the cgroup2 filesystem.
// TODO: move out from talpid crate
pub const CGROUP2_MOUNT_PATH: &str = "/run/mullvad-vpn-cgroups";

/// Identifies packets coming from the cgroup.
/// This should be an arbitrary but unique integer.
pub const NET_CLS_CLASSID: u32 = 0x4d9f41;
/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
/// TODO: what is this used for??
pub const MARK: i32 = 0xf41;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
#[error("split-tunnelinng cgroups v2 error: {0}")]
pub struct Error(#[from] anyhow::Error);

/// Manages PIDs in the linux cgroup2 excluded from the VPN tunnel.
///
/// It's recommended to read the kernel docs before delving into this module:
/// https://docs.kernel.org/admin-guide/cgroup-v2.html
pub struct PidManager {
    inner: Result<Inner, Error>,
}

struct Inner {
    root_cgroup2: Cgroup2,
    excluded_cgroup2: Cgroup2,
}

/// A handle to a cgroup2
///
/// The cgroup is unmounted when droppped.
struct Cgroup2 {
    /// Absolute path of the cgroup2, e.g. `/run/my_cgroup2_mount/my_cgroup2`
    path: PathBuf,

    /// `cgroup.procs` is used to add and list PIDs in the cgroup2.
    procs: File,
}

impl PidManager {
    fn new() -> Self {
        let inner = Self::new_inner().inspect_err(|e| {
            log::error!("Failed to initialize split-tunneling: {e:#?}");
        });

        PidManager { inner }
    }

    fn new_inner() -> Result<Inner, Error> {
        let root_cgroup2 = mount_cgroup2_fs()?;
        let cgroup = SPLIT_TUNNEL_CGROUP_NAME;
        let excluded_cgroup2 = root_cgroup2.create_or_open_child(cgroup)?;

        Ok(Inner {
            root_cgroup2,
            excluded_cgroup2,
        })
    }

    /// Add a PID to the Cgroup to have it excluded from the tunnel.
    pub fn add(&self, pid: pid_t) -> Result<(), Error> {
        let pid = Pid::from_raw(pid);
        self.inner()?.excluded_cgroup2.add_pid(pid)
    }

    /// Remove a PID from the Cgroup to have it included in the tunnel.
    pub fn remove(&self, pid: pid_t) -> Result<(), Error> {
        let pid = Pid::from_raw(pid);
        self.inner()?.root_cgroup2.add_pid(pid)
    }

    /// Return a list of all PIDs currently in the Cgroup excluded from the tunnel.
    pub fn list(&self) -> Result<Vec<pid_t>, Error> {
        self.inner()?.excluded_cgroup2.list_pids()
    }

    /// Removes all PIDs from the Cgroup.
    pub fn clear(&self) -> Result<(), Error> {
        let mut pids = self.list()?;
        while !pids.is_empty() {
            for pid in pids {
                self.remove(pid)?;
            }
            pids = self.list()?;
        }
        Ok(())
    }

    /// Return whether it is enabled
    pub fn is_enabled(&self) -> bool {
        matches!(self.inner, Ok(..))
    }

    fn inner(&self) -> Result<&Inner, Error> {
        self.inner
            .as_ref()
            .ok()
            .context("Split-tunneling is not available")
            .map_err(Into::into)
    }
}

impl Default for PidManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Cgroup2 {
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

        Ok(Cgroup2 { path, procs })
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

    /// Assign a process to this cgroup2.
    fn add_pid(&self, pid: Pid) -> Result<(), Error> {
        // Format the pid as a string
        let mut pid_buf = [0u8; 16];
        write!(&mut pid_buf[..], "{pid}").expect("buf is large enough");
        let pid_str = CStr::from_bytes_until_nul(&pid_buf).expect("buf contains null");

        // Write te pid to `cgroup.procs`.
        nix::unistd::write(&self.procs, pid_str.to_bytes())
            .with_context(|| anyhow!("Failed to add process {pid} to cgroup2"))?;

        Ok(())
    }

    // TODO: should probably be &mut self since we mutate `self.file`
    fn list_pids(&self) -> Result<Vec<pid_t>, Error> {
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
            //.map(Pid::from_raw)
            .collect();
        Ok(pids)
    }
}

impl Drop for Cgroup2 {
    fn drop(&mut self) {
        if let Err(err) = unmount_cgroup2_fs(self) {
            log::error!("{err}");
        };
    }
}

/// Mount the root cgroup2 at [CGROUP2_MOUNT_PATH].
///
/// Returns the root [Cgroup2].
fn mount_cgroup2_fs() -> Result<Cgroup2, Error> {
    let cgroup2_root = Path::new(CGROUP2_MOUNT_PATH);

    // TODO: dedup
    match nix::unistd::mkdir(cgroup2_root, nix::sys::stat::Mode::empty()) {
        Ok(_) | Err(Errno::EEXIST) => {}
        Err(e) => Err(e).context("Failed to create cgroup2")?,
    }

    // `mount -t cgroup2 none <cgroup2_root>`
    // note that this succeeds if the dir is already mounted
    nix::mount::mount(
        None::<&str>,
        cgroup2_root,
        Some("cgroup2"),
        nix::mount::MsFlags::empty(),
        None::<&str>,
    )
    .context("Failed to mount cgroup2 fs")?;

    Cgroup2::open(cgroup2_root)
}

/// Unmount the root cgroup2 at [CGROUP2_MOUNT_PATH].
///
/// `cgroup` will have been unmounted when this function returns.
//
// TODO: Do we need to migrate all processes in this cgroup before removing it?
// v1 implemenatation did this by simply propagating all spawned processes into the cgroup's
// parent, which seems a bit hacky. But maybe it works here as well :shrug: Would be nice to know
// for sure.
fn unmount_cgroup2_fs(cgroup: &Cgroup2) -> Result<(), Error> {
    std::fs::remove_dir(&cgroup.path)
        .context(format!("{cgroup}", cgroup = cgroup.path.display()))
        .context("Failed to unmount cgroup2 fs")
        .map_err(Error)
}
