use mockito::{Mock, ServerGuard};
use std::ffi::{CStr, c_char, c_void};

#[repr(C)]
pub struct SwiftServerMock {
    server_ptr: *const c_void,
    mocks_ptr: *const c_void,
    mocks_len: usize,
    mocks_capacity: usize,
    port: u16,
}

impl SwiftServerMock {
    pub fn new(server: ServerGuard, mocks: Vec<Mock>) -> SwiftServerMock {
        let port = server.socket_address().port();
        let server_ptr = Box::into_raw(Box::new(server)) as *const c_void;
        let (mocks_ptr, mocks_len, mocks_capacity) = mocks.into_raw_parts();

        SwiftServerMock {
            server_ptr,
            mocks_ptr: mocks_ptr.cast(),
            mocks_len,
            mocks_capacity,
            port,
        }
    }
}

#[repr(C)]
pub struct MockEndpoint {
    pub path: *const c_char,
    pub response_code: usize,
    pub response_body: *mut i8,
}
/// # Safety
///
/// `path` must be a pointer to a null terminated string representing the url path.
///
/// `response_code` must be a usize representing the http response code.
///
/// `response_body` must be a pointer to a null terminated string representing the body.
///
/// This must be used in conjuction with [mullvad_api_mock_get] and subsequently [mullvad_api_mock_drop]
/// to make sure it is properly dropped.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn create_mock_endpoint(
    path: *const c_char,
    response_code: usize,
    response_body: *const i8,
) -> MockEndpoint {
    // SAFETY: See notes above
    let path = unsafe { CStr::from_ptr(path).to_owned().into_raw() };
    // SAFETY: See notes above
    let response_body = unsafe { CStr::from_ptr(response_body).to_owned().into_raw() };
    MockEndpoint {
        path,
        response_code,
        response_body,
    }
}

/// # Safety
///
/// `endpoints` must be a pointer to an array of [MockEndpoint]s.
///
/// `endpoint_count` must be a usize matching the length of the array
///
/// Each [MockEndpoint] must fulfill the following:
/// - `path` must be a pointer to a null terminated string representing the url path.
///
/// - `response_code` must be a usize representing the http response code.
///
/// - `response_body` must be a pointer to a null terminated string representing the body.
///
/// This function is safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_get(
    endpoints: *const MockEndpoint,
    endpoint_count: usize,
) -> SwiftServerMock {
    // SAFETY: See notes above
    let slice = unsafe { std::slice::from_raw_parts(endpoints, endpoint_count) };
    let mut server = mockito::Server::new();
    let mut mocks = Vec::new();
    for endpoint in slice {
        // SAFETY: See notes above
        let path = unsafe { std::ffi::CStr::from_ptr(endpoint.path.cast()) }
            .to_str()
            .unwrap();
        // SAFETY: See notes above
        let response_body = unsafe { std::ffi::CStr::from_ptr(endpoint.response_body.cast()) }
            .to_str()
            .unwrap();
        let mock = server
            .mock("GET", path)
            .with_header("content-type", "application/json")
            .with_status(endpoint.response_code)
            .with_body(response_body)
            .create();
        mocks.push(mock);
    }

    SwiftServerMock::new(server, mocks)
}

/// # Safety
///
/// `path` must be a pointer to a null terminated string representing the url path.
///
/// `response_code` must be a usize representing the http response code.
///
/// `match_body` must be a pointer to a null terminated json string representing the body the server expects.
///
/// This function is safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_post(
    path: *const c_char,
    response_code: usize,
    match_body: *const c_char,
) -> SwiftServerMock {
    // SAFETY: See notes above
    let path = unsafe { std::ffi::CStr::from_ptr(path.cast()) }
        .to_str()
        .unwrap();
    // SAFETY: See notes above
    let match_body = unsafe { std::ffi::CStr::from_ptr(match_body.cast()) }
        .to_str()
        .unwrap();
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", path)
        .with_header("content-type", "application/json")
        .with_status(response_code)
        .match_body(mockito::Matcher::JsonString(match_body.to_string()))
        .create();
    SwiftServerMock::new(server, vec![mock])
}

/// Called by the Swift side to signal that the Rust `SwiftServerMock` can be safely
/// dropped from memory.
///
/// # Safety
///
/// `mock_ptr` must be pointing to a valid instance of `SwiftServerMock`. This function
/// is not safe to call multiple times with the same `SwiftServerMock`.
#[unsafe(no_mangle)]
extern "C" fn mullvad_api_mock_drop(mock_ptr: SwiftServerMock) {
    if !mock_ptr.mocks_ptr.is_null() {
        // SAFETY: See notes above
        unsafe {
            drop(Vec::from_raw_parts(
                mock_ptr.mocks_ptr as *mut Mock,
                mock_ptr.mocks_len,
                mock_ptr.mocks_capacity,
            ))
        };
    }
    if !mock_ptr.server_ptr.is_null() {
        // SAFETY: See notes above
        unsafe { drop(Box::from_raw(mock_ptr.server_ptr as *mut ServerGuard)) };
    }
}
