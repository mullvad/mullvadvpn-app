use nix::{errno::Errno, net::if_::if_nametoindex};

/// Converts an interface name into the corresponding index.
pub fn iface_index(name: &str) -> Result<libc::c_uint, IfaceIndexLookupError> {
    if_nametoindex(name).map_err(|error| IfaceIndexLookupError {
        interface_name: name.to_owned(),
        error,
    })
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to get index for interface {interface_name}: {error}")]
pub struct IfaceIndexLookupError {
    pub interface_name: String,
    pub error: Errno,
}
