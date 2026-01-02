//! Linux split-tunneling implementation using cgroups.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

use anyhow::Context;
use libc::pid_t;
use nix::unistd::Pid;
use talpid_cgroup::{
    SPLIT_TUNNEL_CGROUP_NAME,
    v1::{CGroup1, NET_CLS_CLASSID},
    v2::CGroup2,
};

/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
pub const MARK: u32 = 0xf41;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
#[error("Error in split tunneling")]
pub enum Error {
    /// Errors related to split tunneling.
    SplitTunnel(#[from] anyhow::Error),
    /// Errors related to cgroups.
    CGroup(#[from] talpid_cgroup::Error),
}

/// Manages PIDs in the linux cgroup used for vpn tunnel exclusion.
///
/// It's recommended to read the kernel docs before delving into this module:
/// <https://docs.kernel.org/admin-guide/cgroup-v2.html>
pub struct PidManager {
    inner: Result<Inner, Error>,
}

enum Inner {
    CGroup1(InnerCGroup1),
    CGroup2(InnerCGroup2),
}

struct InnerCGroup1 {
    root_cgroup1: CGroup1,
    excluded_cgroup1: CGroup1,
    net_cls_classid: u32,
}

struct InnerCGroup2 {
    root_cgroup2: CGroup2,
    excluded_cgroup2: CGroup2,
}

impl PidManager {
    fn new() -> Self {
        let inner = Self::new_inner();

        if let Err(e) = &inner {
            log::error!("Failed to initialize split-tunneling: {e:?}");
        };

        PidManager { inner }
    }

    fn new_inner() -> Result<Inner, Error> {
        let mut cgroup2_err = None;

        if talpid_cgroup::is_systemd_managed() {
            log::info!("systemd is managing the root cgroup2 on this system");

            // Try to create the cgroup2.
            match Self::new_cgroup2() {
                Ok(inner) => {
                    return Ok(Inner::CGroup2(inner));
                }
                Err(err) => {
                    log::warn!(
                        "Failed to initialize cgroups v2, falling back to cgroup v1 for split tunneling"
                    );
                    log::trace!("{err:?}");
                    cgroup2_err = Some(err);
                }
            }
        } else {
            log::info!(
                "systemd is not managing the root cgroup2 on this system, falling back to cgroups v1 for split tunneling"
            );
        };

        // If it fails, the kernel might be too old, so we fallback on the old cgroup1 solution.
        log::warn!("Note that cgroups v1 is deprecated and will be removed in the future");

        match Self::new_cgroup1() {
            Ok(inner) => Ok(Inner::CGroup1(inner)),
            Err(cgroup1_err) => {
                log::error!("Failed to initialize split-tunneling");
                log::trace!("{cgroup1_err:?}");
                Err(cgroup2_err.unwrap_or(cgroup1_err))
            }
        }
    }

    fn new_cgroup2() -> Result<InnerCGroup2, Error> {
        let root_cgroup2 = CGroup2::open_root()?;

        let excluded_cgroup2 = root_cgroup2.create_or_open_child(SPLIT_TUNNEL_CGROUP_NAME)?;

        Ok(InnerCGroup2 {
            root_cgroup2,
            excluded_cgroup2,
        })
    }

    fn new_cgroup1() -> Result<InnerCGroup1, Error> {
        let root_cgroup = CGroup1::open_root()?;
        let excluded_cgroup = root_cgroup.create_or_open_child(SPLIT_TUNNEL_CGROUP_NAME)?;
        excluded_cgroup.set_net_cls_id(NET_CLS_CLASSID)?;

        Ok(InnerCGroup1 {
            net_cls_classid: NET_CLS_CLASSID,
            root_cgroup1: root_cgroup,
            excluded_cgroup1: excluded_cgroup,
        })
    }

    /// Add a PID to the cgroup2 to have it excluded from the tunnel.
    pub fn add(&self, pid: pid_t) -> Result<(), Error> {
        let pid = Pid::from_raw(pid);
        self.inner()?.add(pid)
    }

    /// Remove a PID from the cgroup2 to have it included in the tunnel.
    pub fn remove(&self, pid: pid_t) -> Result<(), Error> {
        let pid = Pid::from_raw(pid);
        self.inner()?.remove(pid)
    }

    /// Return a list of all PIDs currently in the Cgroup excluded from the tunnel.
    pub fn list(&mut self) -> Result<Vec<pid_t>, Error> {
        self.inner_mut()?.list()
    }

    /// Removes all PIDs from the Cgroup.
    pub fn clear(&mut self) -> Result<(), Error> {
        self.inner_mut()?.clear()
    }

    /// Return whether it is supported/available
    pub fn is_supported(&self) -> bool {
        matches!(self.inner, Ok(..))
    }

    /// Get a handle to the [CGroup2] used for split-tunneling.
    ///
    /// Returns an option if we prevously failed to set up the cgroup2, or if cloning it fails.
    pub fn excluded_cgroup(&self) -> Option<CGroup2> {
        self.inner()
            .ok()?
            .excluded_cgroup()
            .inspect_err(|e| log::error!("Failed to clone file handle to cgroup2: {e}"))
            .ok()?
    }

    /// Get the net_cls classid of the v1 cgroup used for split tunneling.
    ///
    /// This only exist if cgroup v1 is used for split tunneling.
    pub fn net_cls_classid(&self) -> Option<u32> {
        self.inner().ok()?.net_cls_classid()
    }

    fn inner(&self) -> Result<&Inner, Error> {
        self.inner
            .as_ref()
            .ok()
            .context("Split-tunneling is not available")
            .map_err(Into::into)
    }

    fn inner_mut(&mut self) -> Result<&mut Inner, Error> {
        self.inner
            .as_mut()
            .ok()
            .context("Split-tunneling is not available")
            .map_err(Into::into)
    }
}

impl Inner {
    /// Add a PID to the cgroup2 to have it excluded from the tunnel.
    fn add(&self, pid: Pid) -> Result<(), Error> {
        match self {
            Inner::CGroup1(inner) => inner.excluded_cgroup1.add_pid(pid)?,
            Inner::CGroup2(inner) => inner.excluded_cgroup2.add_pid(pid)?,
        }
        Ok(())
    }

    /// Remove a PID from the cgroup to have it included in the tunnel.
    fn remove(&self, pid: Pid) -> Result<(), Error> {
        // PIDs can only be removed from a cgroup by adding them to another cgroup.
        match self {
            Inner::CGroup1(inner) => inner.root_cgroup1.add_pid(pid)?,
            Inner::CGroup2(inner) => inner.root_cgroup2.add_pid(pid)?,
        }
        Ok(())
    }

    /// Return a list of all PIDs currently in the Cgroup excluded from the tunnel.
    fn list(&mut self) -> Result<Vec<pid_t>, Error> {
        Ok(match self {
            Inner::CGroup1(inner) => inner.excluded_cgroup1.list_pids()?,
            Inner::CGroup2(inner) => inner.excluded_cgroup2.list_pids()?,
        })
    }

    /// Removes all PIDs from the Cgroup.
    fn clear(&mut self) -> Result<(), Error> {
        let mut pids = self.list()?;
        while !pids.is_empty() {
            for pid in pids {
                let pid = Pid::from_raw(pid);
                self.remove(pid)?;
            }
            pids = self.list()?;
        }
        Ok(())
    }

    /// Get a handle to the [CGroup2] used for split-tunneling, if any.
    ///
    /// Returns an error if cloning the cgroup fails.
    fn excluded_cgroup(&self) -> Result<Option<CGroup2>, Error> {
        match self {
            Inner::CGroup1(..) => Ok(None),
            Inner::CGroup2(inner) => Ok(inner.excluded_cgroup2.try_clone().map(Some)?),
        }
    }

    /// Get the net_cls classid associated with the v1 cgroup used for split-tunneling, if any.
    ///
    /// This returns none if we're using cgroups v1, or if we failed to create the v1 cgroup.
    fn net_cls_classid(&self) -> Option<u32> {
        match self {
            Inner::CGroup1(inner) => Some(inner.net_cls_classid),
            Inner::CGroup2(..) => None,
        }
    }
}

impl Default for PidManager {
    fn default() -> Self {
        Self::new()
    }
}
