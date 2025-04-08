use std::{
    ffi::CString,
    ptr::{self, null_mut},
};

use mullvad_api::{
    rest::{self, Response},
    RelayListProxy, StatusCode,
};

#[repr(C)]
pub struct SwiftMullvadApiResponse {
    body: *mut u8,
    body_size: usize,
    etag: *mut u8,
    status_code: u16,
    error_description: *mut u8,
    server_response_code: *mut u8,
    success: bool,
}

impl SwiftMullvadApiResponse {
    pub async fn with_body(response: Response<hyper::body::Incoming>) -> Result<Self, rest::Error> {
        let maybe_etag = RelayListProxy::extract_etag(&response);

        let status_code: u16 = response.status().into();
        let body: Vec<u8> = response.body().await?;

        let body_size = body.len();
        let body = body.into_boxed_slice();

        let etag = match maybe_etag {
            Some(etag) => {
                let header_value =
                    CString::new(etag).map_err(|_| rest::Error::InvalidHeaderError)?;
                header_value.into_raw().cast()
            }
            None => ptr::null_mut(),
        };

        Ok(Self {
            body: Box::<[u8]>::into_raw(body).cast(),
            body_size,
            etag,
            status_code,
            error_description: null_mut(),
            server_response_code: null_mut(),
            success: true,
        })
    }

    pub fn ok() -> Self {
        Self {
            success: true,
            error_description: null_mut(),
            body: null_mut(),
            body_size: 0,
            etag: null_mut(),
            status_code: StatusCode::NO_CONTENT.as_u16(),
            server_response_code: null_mut(),
        }
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
            etag: null_mut(),
            status_code,
            error_description,
            server_response_code,
            success: false,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            success: false,
            error_description: c"Request was cancelled".to_owned().into_raw().cast(),
            body: null_mut(),
            body_size: 0,
            etag: null_mut(),
            status_code: 0,
            server_response_code: null_mut(),
        }
    }

    pub fn no_tokio_runtime() -> Self {
        Self {
            success: false,
            error_description: c"Failed to get Tokio runtime".to_owned().into_raw().cast(),
            body: null_mut(),
            body_size: 0,
            etag: null_mut(),
            status_code: 0,
            server_response_code: null_mut(),
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

    if !response.etag.is_null() {
        let _ = CString::from_raw(response.etag.cast());
    }

    if !response.error_description.is_null() {
        let _ = CString::from_raw(response.error_description.cast());
    }

    if !response.server_response_code.is_null() {
        let _ = CString::from_raw(response.server_response_code.cast());
    }
}
