use std::{
    ffi::{self, CString},
    io,
};

/// Converts an interface name into the corresponding index.
pub fn iface_index(name: &str) -> Result<libc::c_uint, IfaceIndexLookupError> {
    let c_name = CString::new(name)
        .map_err(|e| IfaceIndexLookupError::InvalidInterfaceName(name.to_owned(), e))?;
    let index = unsafe { libc::if_nametoindex(c_name.as_ptr()) };
    if index == 0 {
        Err(IfaceIndexLookupError::InterfaceLookupError(
            name.to_owned(),
            io::Error::last_os_error(),
        ))
    } else {
        Ok(index)
    }
}

#[derive(Debug, err_derive::Error)]
pub enum IfaceIndexLookupError {
    #[error(display = "Invalid network interface name: {}", _0)]
    InvalidInterfaceName(String, #[error(source)] ffi::NulError),
    #[error(display = "Failed to get index for interface {}", _0)]
    InterfaceLookupError(String, #[error(source)] io::Error),
}

// b"mole" is [ 0x6d, 0x6f 0x6c, 0x65 ]
pub const TUNNEL_FW_MARK: u32 = 0x6d6f6c65;

pub mod network_manager {
    use dbus::ffidisp::{stdintf::*, Connection};

    const NM_BUS: &str = "org.freedesktop.NetworkManager";
    const NM_TOP_OBJECT: &str = "org.freedesktop.NetworkManager";
    const NM_OBJECT_PATH: &str = "/org/freedesktop/NetworkManager";
    const CONNECTIVITY_CHECK_KEY: &str = "ConnectivityCheckEnabled";
    const RPC_TIMEOUT_MS: i32 = 1500;

    /// Ensures NetworkManager's connectivity check is disabled and returns the connectivity check
    /// previous state. Returns true only if the connectivity check was enabled and is now
    /// disabled. Disabling the connectivity check should be done before a firewall is applied
    /// due to the fact that blocking DNS requests can make it hang:
    /// https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/issues/404
    pub fn nm_disable_connectivity_check(connection: &Connection) -> Option<bool> {
        let nm_manager = connection.with_path(NM_BUS, NM_OBJECT_PATH, RPC_TIMEOUT_MS);
        match nm_manager.get(NM_TOP_OBJECT, CONNECTIVITY_CHECK_KEY) {
            Ok(true) => {
                if let Err(err) = nm_manager.set(NM_TOP_OBJECT, CONNECTIVITY_CHECK_KEY, false) {
                    log::error!(
                        "Failed to disable NetworkManager connectivity check: {}",
                        err
                    );
                    Some(false)
                } else {
                    Some(true)
                }
            }
            Ok(false) => Some(false),
            Err(_) => None,
        }
    }

    /// Enabled NetworkManager's connectivity check. Fails silently.
    pub fn nm_enable_connectivity_check(connection: &Connection) {
        if let Err(err) = connection
            .with_path(NM_BUS, NM_OBJECT_PATH, RPC_TIMEOUT_MS)
            .set(NM_TOP_OBJECT, CONNECTIVITY_CHECK_KEY, true)
        {
            log::error!("Failed to reset NetworkManager connectivity check: {}", err);
        }
    }
}
