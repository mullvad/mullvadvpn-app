//! NSIS plugin DLL (`mullvad_nsis.dll`) for the Mullvad VPN installer.
//!
//! The plugin functions are grouped by module by what they touch - registry,
//! environment PATH, on-disk cleanup, and installer logging. NSIS sees them as
//! flat exports of `mullvad_nsis.dll` (`mullvad_nsis::FunctionName` from NSIS
//! script).

#![cfg(all(target_arch = "x86", target_os = "windows"))]

mod cleanup;
mod handle;
mod logger;
mod pathedit;
mod registry;

/// NSIS status codes returned to the installer scripts.
#[derive(Clone, Copy)]
#[repr(i32)]
pub(crate) enum NsisStatus {
    GeneralError = 0,
    Success = 1,
    FileExists = 2,
    Cancelled = 3,
}
