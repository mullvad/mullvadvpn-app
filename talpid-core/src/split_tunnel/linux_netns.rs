// TODO: "mark"? mark them for what?
// TODO: duplicated from cgroups implementation
/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
pub const MARK: i32 = 0xf41;

/// Errors related to split tunneling.
#[derive(thiserror::Error, Debug)]
#[error("Split-tunneling error")]
pub struct Error(#[source] nullvad::Error);

/// Manages PIDs in the Linux Cgroup excluded from the VPN tunnel.
pub struct PidManager {
    result: nullvad::Result<()>,
}

impl PidManager {
    /// Set up network namespace used for split tunneling.
    pub async fn new() -> Self {
        // if the namespace already exists, clean it up first.
        let _ = nullvad::destroy_namespace();
        let _ = nullvad::nft::remove_nft_rules();

        Self {
            result: nullvad::up().await,
        }
    }

    /// Add a PID to the network namespace to have it excluded from the tunnel.
    pub fn add(&self, pid: i32) -> Result<(), Error> {
        log::warn!("split tunneling not implemented");
        Ok(())
    }

    /// Remove a PID from the network namespace to have it included in the tunnel.
    pub fn remove(&self, pid: i32) -> Result<(), Error> {
        log::warn!("split tunneling not implemented");
        Ok(())
    }

    /// Return a list of all PIDs currently in the network namespace and excluded from the tunnel.
    pub fn list(&self) -> Result<Vec<i32>, Error> {
        log::warn!("split tunneling not implemented");
        Ok(vec![])
    }

    /// Removes all PIDs from the network namespace.
    pub fn clear(&self) -> Result<(), Error> {
        log::warn!("split tunneling not implemented");
        Ok(())
    }

    /// Return whether it is enabled
    pub fn is_enabled(&self) -> bool {
        self.result.is_ok()
    }
}

impl Drop for PidManager {
    fn drop(&mut self) {
        log::info!("Removing split-tunneling network namespace");
        if let Err(e) = nullvad::destroy_namespace() {
            log::error!("{e:#?}");
        }
        if let Err(e) = nullvad::nft::remove_nft_rules() {
            log::error!("{e:#?}");
        }
    }
}
