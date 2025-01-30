use std::{ffi::CStr, ptr::null_mut};


use mullvad_api::rest::Response;

use super::Error;

#[repr(C)]
pub struct SwiftMullvadApiResponse {
    body: *mut u8,
    body_size: usize,
    status_code: u16,
    error_description: *mut u8,
    success: bool,
    should_retry: bool,
    retry_after: u64,
}
impl SwiftMullvadApiResponse {
    pub async fn with_body(response: Response<hyper::body::Incoming>) -> Result<Self, Error> {
        let status_code: u16 = response.status().into();
        let body: Vec<u8> = response.body().await.map_err(Error::Rest)?;

        let body_size = body.len();
        let body = body.into_boxed_slice();

        Ok(Self {
            body: Box::<[u8]>::into_raw(body).cast(),
            body_size,
            status_code,
            error_description: null_mut(),
            success: true,
            should_retry: false,
            retry_after: 0,
        })
    }

    pub fn error() -> Self {
        Self {
            body: null_mut(),
            body_size: 0,
            status_code: 0,
            error_description: null_mut(),
            success: false,
            should_retry: false,
            retry_after: 0,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_response_drop(response: SwiftMullvadApiResponse) {
    if !response.body.is_null() {
        let _ = Vec::from_raw_parts(response.body, response.body_size, response.body_size);
    }

    if !response.error_description.is_null() {
        let _ = CStr::from_ptr(response.error_description.cast());
    }
}

