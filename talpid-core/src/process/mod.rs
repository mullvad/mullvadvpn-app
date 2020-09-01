/// A module for all OpenVPN related process management.
#[cfg(not(target_os = "android"))]
pub mod openvpn;

/// A trait for stopping subprocesses gracefully.
pub mod stoppable_process;
