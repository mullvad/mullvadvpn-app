//! Errors of the Windows firewall implementation.

use std::io;

use windows_sys::Win32::Foundation::FWP_E_TIMEOUT;

/// Errors that can happen when initializing or deinitializing the firewall.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to initialize the firewall module
    #[error("Failed to initialize windows firewall module")]
    Initialization(#[source] io::Error),

    /// Failure to deinitialize the firewall module
    #[error("Failed to deinitialize windows firewall module")]
    Deinitialization(#[source] io::Error),
}

/// Errors that can happen when applying a firewall policy or resetting it.
#[derive(thiserror::Error, Debug)]
pub enum ApplyError {
    /// The policy could not be applied
    #[error("Failed to apply firewall policy")]
    Generic(#[source] io::Error),

    /// The WFP transaction lock could not be acquired in time
    #[error("Timed out waiting for the WFP transaction lock")]
    LockTimeout(#[source] io::Error),
}

impl ApplyError {
    /// Convert a WFP I/O error, separating out the transaction-lock timeout the
    /// daemon reports to the user interface.
    pub fn from_io(error: io::Error) -> Self {
        if error.raw_os_error() == Some(FWP_E_TIMEOUT) {
            Self::LockTimeout(error)
        } else {
            Self::Generic(error)
        }
    }
}
