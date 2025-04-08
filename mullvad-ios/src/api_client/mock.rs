use mockito::{Mock, ServerGuard};
use std::ffi::c_void;

#[repr(C)]
pub struct SwiftServerMock {
    server_ptr: *const c_void,
    mock_ptr: *const c_void,
    port: u16,
}

impl SwiftServerMock {
    pub fn new(server: ServerGuard, mock: Mock) -> SwiftServerMock {
        let port = server.socket_address().port();
        let server_ptr = Box::into_raw(Box::new(server)) as *const c_void;
        let mock_ptr = Box::into_raw(Box::new(mock)) as *const c_void;

        SwiftServerMock {
            server_ptr,
            mock_ptr,
            port,
        }
    }
}

/// # Safety
///
/// `method` must be a pointer to a null terminated string representing the http method.
///
/// `path` must be a pointer to a null terminated string representing the url path.
///
/// `response_code` must be a usize representing the http response code.
///
/// `response_body` must be a pointer to a null terminated string representing the body.
///
/// This function is safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_get(
    path: *const u8,
    response_code: usize,
    response_body: *const u8,
) -> SwiftServerMock {
    let path = unsafe { std::ffi::CStr::from_ptr(path.cast()) }
        .to_str()
        .unwrap();
    let response_body = unsafe { std::ffi::CStr::from_ptr(response_body.cast()) }
        .to_str()
        .unwrap();
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", path)
        .with_header("content-type", "application/json")
        .with_status(response_code)
        .with_body(response_body)
        .create();
    SwiftServerMock::new(server, mock)
}

/// # Safety
///
/// `path` must be a pointer to a null terminated string representing the url path.
///
/// `response_code` must be a usize representing the http response code.
///
/// `match_body` must be a pointer to a null terminated string representing the body the server expects.
///
/// This function is safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_post(
    path: *const u8,
    response_code: usize,
    match_body: *const u8,
) -> SwiftServerMock {
    let path = unsafe { std::ffi::CStr::from_ptr(path.cast()) }
        .to_str()
        .unwrap();
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
    SwiftServerMock::new(server, mock)
}

/// Called by the Swift side to signal that the Rust `SwiftServerMock` can be safely
/// dropped from memory.
///
/// # Safety
///
/// `mock_ptr` must be pointing to a valid instance of `SwiftServerMock`. This function
/// is not safe to call multiple times with the same `SwiftServerMock`.
#[no_mangle]
extern "C" fn mullvad_api_mock_drop(mock_ptr: SwiftServerMock) {
    if !mock_ptr.mock_ptr.is_null() {
        unsafe { drop(Box::from_raw(mock_ptr.mock_ptr as *mut Mock)) };
    }
    if !mock_ptr.server_ptr.is_null() {
        unsafe { drop(Box::from_raw(mock_ptr.server_ptr as *mut ServerGuard)) };
    }
}
