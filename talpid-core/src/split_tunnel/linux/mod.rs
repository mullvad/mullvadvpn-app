//! Linux split-tunneling implementation using cgroup2.
//!
//! It's recommended to read the kernel docs before delving into this module:
//! <https://docs.kernel.org/admin-guide/cgroup-v2.html>

use anyhow::{Context, anyhow};
use libc::pid_t;
use nftnl::{Batch, Chain, Hook, MsgType, Policy, ProtoFamily, Rule, Table, nft_expr};
use nix::unistd::Pid;
use talpid_types::cgroup::{CGROUP2_DEFAULT_MOUNT_PATH, SPLIT_TUNNEL_CGROUP_NAME};

mod cgroups_v1;
mod cgroups_v2;

use crate::{
    firewall,
    split_tunnel::linux::cgroups_v1::{DEFAULT_NET_CLS_DIR, NET_CLS_CLASSID},
};
pub use cgroups_v1::CGroup1;
pub use cgroups_v2::CGroup2;

/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
pub const MARK: u32 = 0xf41;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
#[error("Error in split tunneling")]
pub struct Error(#[from] anyhow::Error);

/// Manages PIDs in the linux cgroup2 excluded from the VPN tunnel.
///
/// It's recommended to read the kernel docs before delving into this module:
/// https://docs.kernel.org/admin-guide/cgroup-v2.html
pub struct PidManager {
    inner: Result<Inner, Error>,
}

enum Inner {
    CGroup1(InnerCGroup1),
    CGroup2(InnerCGroup2),
}

struct InnerCGroup1 {
    root_cgroup: CGroup1,
    excluded_cgroup: CGroup1,
    net_cls: u32,
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
        // Try to create the cgroup2.
        let inner = match Self::new_cgroup2() {
            Ok(inner) => Inner::CGroup2(inner),
            Err(cgroup2_err) => {
                // If it does not success, the kernel might be too old, so we fallback on the old cgroup1 solution.
                match Self::new_cgroup1() {
                    Ok(inner) => Inner::CGroup1(inner),
                    Err(cgroup1_err) => {
                        log::error!(
                            "Failed to initialize split-tunneling: {cgroup2_err:?}, {cgroup1_err:?}"
                        );
                        // TODO: Should we try to compose these errors?
                        return Err(cgroup2_err);
                    }
                }
            } // oh god the indentation ..
        };
        Ok(inner)
    }

    fn new_cgroup2() -> Result<InnerCGroup2, Error> {
        let root_cgroup2 =
            CGroup2::open(CGROUP2_DEFAULT_MOUNT_PATH).context("Failed to open root cgroup2")?;

        let excluded_cgroup2 = root_cgroup2.create_or_open_child(SPLIT_TUNNEL_CGROUP_NAME)?;

        assert_nft_supports_cgroup2(&excluded_cgroup2)
            .context("cgroup2 not supported by nftables, are you running an old kernel?")?;

        Ok(InnerCGroup2 {
            root_cgroup2,
            excluded_cgroup2,
        })
    }

    fn new_cgroup1() -> Result<InnerCGroup1, Error> {
        let root_cgroup = CGroup1::open(DEFAULT_NET_CLS_DIR)?;
        let excluded_cgroup = root_cgroup.create_or_open_child(SPLIT_TUNNEL_CGROUP_NAME)?;
        excluded_cgroup.set_net_cls_id(NET_CLS_CLASSID)?;

        Ok(InnerCGroup1 {
            net_cls: NET_CLS_CLASSID, // TODO: set this
            root_cgroup,
            excluded_cgroup,
        })
    }

    /// Add a PID to the cgroup2 to have it excluded from the tunnel.
    pub fn add(&self, pid: pid_t) -> Result<(), Error> {
        let pid = Pid::from_raw(pid);
        self.inner()?.add(pid)
    }

    /// Remove a PID from the cgroup2 to have it included in the tunnel.
    pub fn remove(&self, pid: pid_t) -> Result<(), Error> {
        // PIDs can only be removed from a cgroup2 by adding them to another cgroup2.
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
            Inner::CGroup1(inner) => inner.excluded_cgroup.add_pid(pid),
            Inner::CGroup2(inner) => inner.excluded_cgroup2.add_pid(pid),
        }
    }

    /// Remove a PID from the cgroup to have it included in the tunnel.
    fn remove(&self, pid: Pid) -> Result<(), Error> {
        // PIDs can only be removed from a cgroup by adding them to another cgroup.
        match self {
            Inner::CGroup1(inner) => inner.root_cgroup.add_pid(pid),
            Inner::CGroup2(inner) => inner.root_cgroup2.add_pid(pid),
        }
    }

    /// Return a list of all PIDs currently in the Cgroup excluded from the tunnel.
    fn list(&mut self) -> Result<Vec<pid_t>, Error> {
        match self {
            Inner::CGroup1(inner) => inner.excluded_cgroup.list_pids(),
            Inner::CGroup2(inner) => inner.excluded_cgroup2.list_pids(),
        }
    }

    /// Removes all PIDs from the Cgroup.
    fn clear(&mut self) -> Result<(), Error> {
        let mut pids = self.list()?;
        while !pids.is_empty() {
            for pid in pids {
                // TODO: We didn't do this before. Should be fine.
                let pid = Pid::from_raw(pid);
                self.remove(pid)?;
            }
            pids = self.list()?;
        }
        Ok(())
    }

    /// Get a handle to the [CGroup2] used for split-tunneling.
    ///
    /// Returns an error if we prevously failed to set up the cgroup2, or if cloning it fails.
    fn excluded_cgroup(&self) -> Result<CGroup2, Error> {
        match self {
            Inner::CGroup1(..) => Err(anyhow!("blahaha").into()),
            Inner::CGroup2(inner) => inner.excluded_cgroup2.try_clone(),
        }
    }

    /// Try to clone the cgroup2 handle.
    ///
    /// This is fallible because cloning file descriptors can fail.
    pub fn try_clone(&self) -> Result<Self, Error> {
        match self {
            Inner::CGroup1(inner_cgroup1) => todo!(),
            Inner::CGroup2(inner) => inner.try_clone().map(Inner::CGroup2),
        }
    }
}

impl InnerCGroup2 {
    /// Try to clone the cgroup2 handle.
    ///
    /// This is fallible because cloning file descriptors can fail.
    pub fn try_clone(&self) -> Result<Self, Error> {
        Ok(Self {
            root_cgroup2: self.root_cgroup2.try_clone()?,
            excluded_cgroup2: self.excluded_cgroup2.try_clone()?,
        })
    }
}

/// Check whether we can create an nft table with a `socket cgroupv2 level x` rule.
///
/// Assuming that this process has the sufficient privileges, then this function should only fail
/// when the kernel doesn't support this kind of rule. This is the case for kernels predating 5.12.
// TODO: this fugly bc
// - 1. we're doing firewall stuff outside the firewall module
// - 2. we're creating an invariant that this nft expression is the same as the one we create in the firewall module
fn assert_nft_supports_cgroup2(cgroup: &CGroup2) -> Result<(), Error> {
    let table_name = c"mullvad-test-cgroup2-capability";

    let mut batch = Batch::new();
    let table = Table::new(table_name, ProtoFamily::Inet);
    batch.add(&table, MsgType::Add);

    let mut chain = Chain::new(c"test", &table);
    chain.set_hook(Hook::Out, 0);
    chain.set_policy(Policy::Accept);
    batch.add(&chain, MsgType::Add);

    let mut rule = Rule::new(&chain);
    rule.add_expr(&nft_expr!(socket cgroupv2 level 1));
    rule.add_expr(&nft_expr!(cmp == cgroup.inode()));
    rule.add_expr(&nft_expr!(verdict accept));
    batch.add(&rule, MsgType::Add);

    // Remove the table. Since this happens is the same batch, the table will never process any packets.
    // This makes it effectively a dry-run.
    let table = Table::new(table_name, ProtoFamily::Inet);
    batch.add(&table, MsgType::Del);

    let batch = batch.finalize();
    firewall::linux::Firewall::send_and_process(&batch)
        .context("Failed to add nft cgroupv2 rule")?;

    Ok(())
}

impl Default for PidManager {
    fn default() -> Self {
        Self::new()
    }
}
