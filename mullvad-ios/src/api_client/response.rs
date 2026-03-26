use std::{
    ffi::{CString, c_char},
    ptr::null_mut,
};

use mullvad_api::{
    StatusCode,
    relay_list_transparency::SigsumVerifiedRelayList,
    rest::{self, Response},
};

#[repr(C)]
pub struct SwiftMullvadApiResponse {
    body: *mut u8,
    body_size: usize,
    etag: *mut c_char,
    status_code: u16,
    error_description: *mut c_char,
    server_response_code: *mut c_char,
    success: bool,
    sigsum_timestamp: i64,
    sigsum_digest: *mut c_char,
}

impl SwiftMullvadApiResponse {
    pub async fn with_body(response: Response<hyper::body::Incoming>) -> Result<Self, rest::Error> {
        // let maybe_etag = None;

        let status_code: u16 = response.status().into();
        let body: Vec<u8> = response.body().await?;

        let body_size = body.len();
        let body = body.into_boxed_slice();

        Ok(Self {
            body: Box::<[u8]>::into_raw(body).cast(),
            body_size,
            etag: null_mut(),
            status_code,
            error_description: null_mut(),
            server_response_code: null_mut(),
            success: true,
            sigsum_timestamp: 0,
            sigsum_digest: null_mut(),
        })
    }

    pub fn with_sigsum_verified_body(
        sigsum_payload: Option<SigsumVerifiedRelayList>,
    ) -> Result<Self, rest::Error> {
        let (body, body_size, sigsum_timestamp, sigsum_digest) = match sigsum_payload {
            Some(SigsumVerifiedRelayList {
                content,
                digest,
                timestamp,
            }) => {
                let body_size = content.len();
                let body = Box::<[u8]>::into_raw(content.into_boxed_slice()).cast();

                // TODO: consider handling invalid timestamp ? Could we not clamp the time interval
                // down to something that is within 500 years of the unix timestamp?
                // Should we just `.expect("Timestamp more than 500 years away from 1970");`
                let sigsum_timestamp = timestamp.timestamp_nanos_opt().unwrap_or(0);
                let sigsum_digest = CString::new(digest.as_ref())
                    .map_err(|err| {
                        log::error!("Found a nil byte in sigsum_digest string: {err}");
                        rest::Error::SigsumDeserializeError
                    })?
                    .into_raw();

                (body, body_size, sigsum_timestamp, sigsum_digest)
            }
            None => (null_mut(), 0, 0, null_mut()),
        };
        Ok(Self {
            success: true,
            error_description: null_mut(),
            body,
            body_size,
            etag: null_mut(),
            status_code: 0,
            server_response_code: null_mut(),
            sigsum_timestamp,
            sigsum_digest,
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
            sigsum_timestamp: 0,
            sigsum_digest: null_mut(),
        }
    }

    pub fn access_method_error(err: mullvad_api::access_mode::Error) -> Self {
        let to_cstr_pointer = |str| {
            CString::new(str)
                .map(|cstr| cstr.into_raw())
                .unwrap_or(null_mut())
        };
        let error_description = to_cstr_pointer(err.to_string());

        Self {
            body: null_mut(),
            body_size: 0,
            etag: null_mut(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            error_description,
            server_response_code: null_mut(),
            success: false,
            sigsum_timestamp: 0,
            sigsum_digest: null_mut(),
        }
    }

    pub fn rest_error(err: mullvad_api::rest::Error) -> Self {
        if err.is_aborted() {
            return Self::cancelled();
        }

        let to_cstr_pointer = |str| {
            CString::new(str)
                .map(|cstr| cstr.into_raw())
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
            sigsum_timestamp: 0,
            sigsum_digest: null_mut(),
        }
    }

    pub fn cancelled() -> Self {
        Self {
            success: false,
            error_description: c"Request was cancelled".to_owned().into_raw(),
            body: null_mut(),
            body_size: 0,
            etag: null_mut(),
            status_code: 0,
            server_response_code: null_mut(),
            sigsum_timestamp: 0,
            sigsum_digest: null_mut(),
        }
    }

    pub fn no_tokio_runtime() -> Self {
        Self {
            success: false,
            error_description: c"Failed to get Tokio runtime".to_owned().into_raw(),
            body: null_mut(),
            body_size: 0,
            etag: null_mut(),
            status_code: 0,
            server_response_code: null_mut(),
            sigsum_timestamp: 0,
            sigsum_digest: null_mut(),
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_response_drop(response: SwiftMullvadApiResponse) {
    unsafe {
        if !response.body.is_null() {
            let _ = Vec::from_raw_parts(response.body, response.body_size, response.body_size);
        }

        if !response.etag.is_null() {
            let _ = CString::from_raw(response.etag);
        }

        if !response.error_description.is_null() {
            let _ = CString::from_raw(response.error_description);
        }

        if !response.server_response_code.is_null() {
            let _ = CString::from_raw(response.server_response_code);
        }

        if !response.sigsum_digest.is_null() {
            let _ = CString::from_raw(response.sigsum_digest);
        }
    }
}
