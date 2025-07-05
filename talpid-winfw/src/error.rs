//! Error type.

use talpid_types::tunnel::FirewallPolicyError;

use crate::sys;

/// Errors that can happen when configuring the Windows firewall.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to initialize windows firewall module
    #[error("Failed to initialize windows firewall module")]
    Initialization,

    /// Failure to deinitialize windows firewall module
    #[error("Failed to deinitialize windows firewall module")]
    Deinitialization,

    /// Failure to apply a firewall _connecting_ policy
    #[error("Failed to apply connecting firewall policy")]
    ApplyingConnectingPolicy(#[source] FirewallPolicyError),

    /// Failure to apply a firewall _connected_ policy
    #[error("Failed to apply connected firewall policy")]
    ApplyingConnectedPolicy(#[source] FirewallPolicyError),

    /// Failure to apply firewall _blocked_ policy
    #[error("Failed to apply blocked firewall policy")]
    ApplyingBlockedPolicy(#[source] FirewallPolicyError),

    /// Failure to reset firewall policies
    #[error("Failed to reset firewall policies")]
    ResettingPolicy(#[source] FirewallPolicyError),
}

impl From<sys::InitializationResult> for Result<(), Error> {
    fn from(result: sys::InitializationResult) -> Self {
        match result.success {
            true => Ok(()),
            false => Err(Error::Initialization),
        }
    }
}

impl From<sys::DeinitializationResult> for Result<(), Error> {
    fn from(result: sys::DeinitializationResult) -> Self {
        match result.success {
            true => Ok(()),
            false => Err(Error::Deinitialization),
        }
    }
}
