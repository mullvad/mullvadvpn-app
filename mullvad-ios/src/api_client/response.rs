use std::{ffi::CString, ptr::null_mut};

use mullvad_api::rest::{self, Response};

#[repr(C)]
pub struct SwiftMullvadApiResponse {
    body: *mut u8,
    body_size: usize,
    status_code: u16,
    error_description: *mut u8,
    server_response_code: *mut u8,
    success: bool,
    should_retry: bool,
    retry_after: u64,
}
impl SwiftMullvadApiResponse {
    pub async fn with_body(response: Response<hyper::body::Incoming>) -> Result<Self, rest::Error> {
        let status_code: u16 = response.status().into();
        let body: Vec<u8> = response.body().await?;

        let body_size = body.len();
        let body = body.into_boxed_slice();

        Ok(Self {
            body: Box::<[u8]>::into_raw(body).cast(),
            body_size,
            status_code,
            error_description: null_mut(),
            server_response_code: null_mut(),
            success: true,
            should_retry: false,
            retry_after: 0,
        })
    }

    pub fn rest_error(err: mullvad_api::rest::Error) -> Self {
        if err.is_aborted() {
            return Self::cancelled();
        }

        let to_cstr_pointer = |str| {
            CString::new(str)
                .map(|cstr| cstr.into_raw().cast())
                .unwrap_or(null_mut())
        };

        let should_retry = err.is_network_error();
        let error_description = to_cstr_pointer(err.to_string());
        let (status_code, server_response_code): (u16, _) =
            if let rest::Error::ApiError(status_code, error_code) = err {
                (status_code.into(), to_cstr_pointer(error_code))
            } else {
                (0, null_mut())
            };

        Self {
            body: null_mut(),
            body_size: 0,
            status_code,
            error_description,
            server_response_code,
            success: false,
            should_retry,
            retry_after: 0,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            success: false,
            should_retry: false,
            error_description: c"Request was cancelled".to_owned().into_raw().cast(),
            body: null_mut(),
            body_size: 0,
            status_code: 0,
            server_response_code: null_mut(),
            retry_after: 0,
        }
    }

    pub fn no_tokio_runtime() -> Self {
        Self {
            success: false,
            should_retry: false,
            error_description: c"Failed to get Tokio runtime".to_owned().into_raw().cast(),
            body: null_mut(),
            body_size: 0,
            status_code: 0,
            server_response_code: null_mut(),
            retry_after: 0,
        }
    }
}

/// Called by the Swift side to signal that the Rust `SwiftMullvadApiResponse` can be safely
/// dropped from memory.
///
/// # Safety
///
/// `response` must be pointing to a valid instance of `SwiftMullvadApiResponse`. This function
/// is not safe to call multiple times with the same `SwiftMullvadApiResponse`.
#[no_mangle]
pub unsafe extern "C" fn mullvad_response_drop(response: SwiftMullvadApiResponse) {
    if !response.body.is_null() {
        let _ = Vec::from_raw_parts(response.body, response.body_size, response.body_size);
    }

    if !response.error_description.is_null() {
        let _ = CString::from_raw(response.error_description.cast());
    }

    if !response.server_response_code.is_null() {
        let _ = CString::from_raw(response.server_response_code.cast());
    }
}
