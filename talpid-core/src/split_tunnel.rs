#![cfg(target_os = "linux")]
use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};
use talpid_types::SPLIT_TUNNEL_CGROUP_NAME;

const NETCLS_DIR: &str = "/sys/fs/cgroup/net_cls/";

/// Identifies packets coming from the cgroup.
/// This should be an arbitrary but unique integer.
pub const NETCLS_CLASSID: u32 = 0x4d9f41;
/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
pub const MARK: i32 = 0xf41;

/// Errors related to split tunneling.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Unable to create cgroup.
    #[error(display = "Unable to initialize net_cls cgroup instance")]
    InitNetClsCGroup(#[error(source)] nix::Error),

    /// Unable to create cgroup.
    #[error(display = "Unable to create cgroup for excluded processes")]
    CreateCGroup(#[error(source)] io::Error),

    /// Unable to set class ID for cgroup.
    #[error(display = "Unable to set cgroup class ID")]
    SetCGroupClassId(#[error(source)] io::Error),

    /// Unable to add PID to cgroup.procs.
    #[error(display = "Unable to add PID to cgroup.procs")]
    AddCGroupPid(#[error(source)] io::Error),

    /// Unable to remove PID to cgroup.procs.
    #[error(display = "Unable to remove PID from cgroup")]
    RemoveCGroupPid(#[error(source)] io::Error),

    /// Unable to read cgroup.procs.
    #[error(display = "Unable to obtain PIDs from cgroup.procs")]
    ListCGroupPids(#[error(source)] io::Error),
}

/// Manages PIDs to exclude from the tunnel.
pub struct PidManager;

impl PidManager {
    /// Create object to manage split-tunnel PIDs.
    pub fn new() -> Result<PidManager, Error> {
        Self::create_cgroup()?;
        Ok(PidManager {})
    }

    /// Set up cgroup used to track PIDs for split tunneling.
    fn create_cgroup() -> Result<(), Error> {
        let netcls_dir = Path::new(NETCLS_DIR);
        if !netcls_dir.exists() {
            fs::create_dir(netcls_dir.clone()).map_err(Error::CreateCGroup)?;

            // https://www.kernel.org/doc/Documentation/cgroup-v1/net_cls.txt
            nix::mount::mount(
                Some("net_cls"),
                netcls_dir,
                Some("cgroup"),
                nix::mount::MsFlags::empty(),
                Some("net_cls"),
            )
            .map_err(Error::InitNetClsCGroup)?;
        }

        let exclusions_dir = netcls_dir.join(SPLIT_TUNNEL_CGROUP_NAME);

        if !exclusions_dir.exists() {
            fs::create_dir(exclusions_dir.clone()).map_err(Error::CreateCGroup)?;
        }

        let classid_path = exclusions_dir.join("net_cls.classid");
        fs::write(classid_path, NETCLS_CLASSID.to_string().as_bytes())
            .map_err(Error::SetCGroupClassId)
    }

    /// Add a PID to exclude from the tunnel.
    pub fn add(&self, pid: i32) -> Result<(), Error> {
        self.add_list(&[pid])
    }

    /// Add PIDs to exclude from the tunnel.
    pub fn add_list(&self, pids: &[i32]) -> Result<(), Error> {
        let exclusions_path = Path::new(NETCLS_DIR)
            .join(SPLIT_TUNNEL_CGROUP_NAME)
            .join("cgroup.procs");

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(exclusions_path)
            .map_err(Error::AddCGroupPid)?;

        let mut writer = BufWriter::new(file);

        for pid in pids {
            writer
                .write_all(pid.to_string().as_bytes())
                .map_err(Error::AddCGroupPid)?;
        }

        Ok(())
    }

    /// Remove a PID from processes to exclude from the tunnel.
    pub fn remove(&self, pid: i32) -> Result<(), Error> {
        // FIXME: We remove PIDs from our cgroup here by adding
        //        them to the parent cgroup. This seems wrong.
        let exclusions_path = Path::new(NETCLS_DIR).join("cgroup.procs");

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(exclusions_path)
            .map_err(Error::RemoveCGroupPid)?;

        file.write_all(pid.to_string().as_bytes())
            .map_err(Error::RemoveCGroupPid)
    }

    /// Return a list of PIDs that are excluded from the tunnel.
    pub fn list(&self) -> Result<Vec<i32>, Error> {
        // TODO: manage child PIDs somehow?

        let exclusions_path = Path::new(NETCLS_DIR)
            .join(SPLIT_TUNNEL_CGROUP_NAME)
            .join("cgroup.procs");

        let file = fs::File::open(exclusions_path).map_err(Error::ListCGroupPids)?;

        let result: Result<Vec<i32>, io::Error> = BufReader::new(file)
            .lines()
            .map(|line| {
                line.and_then(|v| {
                    v.parse()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                })
            })
            .collect();
        result.map_err(Error::ListCGroupPids)
    }

    /// Clear list of PIDs to exclude from the tunnel.
    pub fn clear(&self) -> Result<(), Error> {
        // TODO: reuse file handle
        let pids = self.list()?;

        for pid in pids {
            self.remove(pid)?;
        }

        Ok(())
    }
}
