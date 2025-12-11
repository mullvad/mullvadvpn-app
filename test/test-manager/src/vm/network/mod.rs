use std::net::Ipv4Addr;

#[cfg(target_os = "linux")]
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
    CUSTOM_TUN_LOCAL_TUN_ADDR, CUSTOM_TUN_REMOTE_PUBKEY, CUSTOM_TUN_REMOTE_REAL_PORT,
    CUSTOM_TUN_REMOTE_TUN_ADDR,
};

/// Port on NON_TUN_GATEWAY that hosts a SOCKS5 server
pub const SOCKS5_PORT: u16 = 54321;

/// Get the name of the bridge interface between the test-manager and the test-runner.
pub fn bridge(
    #[cfg(target_os = "macos")] bridge_ip: &Ipv4Addr,
) -> anyhow::Result<(String, Ipv4Addr)> {
    #[cfg(target_os = "macos")]
    {
        crate::vm::network::macos::find_vm_bridge(bridge_ip)
    }
    #[cfg(not(target_os = "macos"))]
    Ok((platform::BRIDGE_NAME.to_owned(), platform::NON_TUN_GATEWAY))
}
