// #[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux as platform;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos as platform;

// Import shared constants and functions
pub use platform::{
    CUSTOM_TUN_GATEWAY, CUSTOM_TUN_INTERFACE_NAME, CUSTOM_TUN_LOCAL_PRIVKEY,
    CUSTOM_TUN_LOCAL_TUN_ADDR, CUSTOM_TUN_REMOTE_PUBKEY, CUSTOM_TUN_REMOTE_REAL_ADDR,
    CUSTOM_TUN_REMOTE_REAL_PORT, CUSTOM_TUN_REMOTE_TUN_ADDR, DUMMY_LAN_INTERFACE_IP,
    NON_TUN_GATEWAY,
};
