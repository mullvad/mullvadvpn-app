use crate::rest;
use std::ffi::CString;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum MullvadApiErrorKind {
    NoError = 0,
    StringParsing = -1,
    SocketAddressParsing = -2,
    AsyncRuntimeInitialization = -3,
    BadResponse = -4,
}

/// MullvadApiErrorKind contains a description and an error kind. If the error kind is
/// `MullvadApiErrorKind` is NoError, the pointer will be nil.
#[repr(C)]
pub struct MullvadApiError {
    description: *mut libc::c_char,
    kind: MullvadApiErrorKind,
}

impl MullvadApiError {
    pub fn new(kind: MullvadApiErrorKind, error: &dyn std::error::Error) -> Self {
        let description = CString::new(format!("{error:?}")).unwrap_or_default();
        Self {
            description: description.into_raw(),
            kind,
        }
    }

    pub fn api_err(error: rest::Error) -> Self {
        Self::new(MullvadApiErrorKind::BadResponse, &error)
    }

    pub fn with_str(kind: MullvadApiErrorKind, description: &str) -> Self {
        let description = CString::new(description).unwrap_or_default();
        Self {
            description: description.into_raw(),
            kind,
        }
    }

    pub fn ok() -> MullvadApiError {
        Self {
            description: std::ptr::null_mut(),
            kind: MullvadApiErrorKind::NoError,
        }
    }

    pub fn drop(self) {
        if self.description.is_null() {
            return;
        }

        let _ = unsafe { CString::from_raw(self.description) };
    }
}

#[no_mangle]
pub extern "C" fn mullvad_api_error_drop(error: MullvadApiError) {
    error.drop()
}
