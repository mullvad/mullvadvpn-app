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

#[derive(Debug, thiserror::Error)]
pub enum IfaceIndexLookupError {
    #[error("Invalid network interface name: {0}")]
    InvalidInterfaceName(String, #[source] ffi::NulError),
    #[error("Failed to get index for interface {0}")]
    InterfaceLookupError(String, #[source] io::Error),
}
