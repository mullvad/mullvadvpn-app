#![cfg(target_os = "ios")]
#![allow(clippy::undocumented_unsafe_blocks)]

mod api_client;
mod encrypted_dns_proxy;
mod ephemeral_peer_proxy;
mod shadowsocks_proxy;
pub mod tunnel_obfuscator_proxy;

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[unsafe(no_mangle)]
pub static CONFIG_SERVICE_PORT: u16 = talpid_tunnel_config_client::CONFIG_SERVICE_PORT;

mod ios {
    use std::sync::OnceLock;
    use tokio::runtime::{Builder, Handle, Runtime};

    static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

    pub fn mullvad_ios_runtime() -> Result<Handle, String> {
        match RUNTIME.get_or_init(|| {
            Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|error| ToString::to_string(&error))
        }) {
            Ok(runtime) => Ok(runtime.handle().clone()),
            Err(error) => Err(error.clone()),
        }
    }
}
use std::{ffi::c_void, sync::Arc};

use mockito::{Mock, Server, ServerOpts};

#[repr(C)]
pub struct SwiftServerMock {
    server_ptr: *const c_void,
    mock_ptr: *const c_void,
    port: u16,
}

impl SwiftServerMock {
    pub fn new(server: Server, mock: Mock) -> SwiftServerMock {
        let port = server.socket_address().port();
        let server_ptr = Box::into_raw(Box::new(server)) as *const c_void;
        let mock_ptr = Box::into_raw(Box::new(mock)) as *const c_void;

        SwiftServerMock {
            server_ptr,
            mock_ptr,
            port,
        }
    }

    pub unsafe fn to_mock(&mut self) -> &Mock {
        return unsafe { &*self.mock_ptr.cast() }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_server_response(
    // server_guard: SwiftServerGuard,
    method: *const u8,
    path: *const u8,
    response_code: usize,
    response_body: *const u8,
) -> SwiftServerMock {
    let method = unsafe { std::ffi::CStr::from_ptr(method.cast()) }
        .to_str()
        .unwrap();
    let path = unsafe { std::ffi::CStr::from_ptr(path.cast()) }
        .to_str()
        .unwrap();
    let response_body = unsafe { std::ffi::CStr::from_ptr(response_body.cast()) }
        .to_str()
        .unwrap();
    let mut server = mockito::Server::new_with_opts(ServerOpts {
        port: 8080,
        ..Default::default()
    });
    let mock = server
        .mock(method, path)
        .with_header("content-type", "application/json")
        .with_status(response_code)
        .with_body(response_body)
        .create();
    SwiftServerMock::new(server, mock)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_got_called(mut server: SwiftServerMock) -> bool {
    server.to_mock().matched()
}

use ios::*;
