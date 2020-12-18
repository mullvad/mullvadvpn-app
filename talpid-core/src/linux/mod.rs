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
pub const TUNNEL_TABLE_ID: u32 = 0x6d6f6c65;
