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
pub struct SwiftServerMock(*const c_void);

impl SwiftServerMock {
    pub fn new(mock: Mock) -> SwiftServerMock {
        SwiftServerMock(Arc::into_raw(Arc::new(mock)) as *const c_void)
    }

    pub unsafe fn to_mock(self) -> Arc<Mock> {
        Arc::increment_strong_count(self.0);
        Arc::from_raw(self.0 as *const Mock)
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
    // let mut server = server_guard.into_rust_server_guard();
    let server = Server::new_with_opts(ServerOpts {
        port: 8080,
        ..Default::default()
    })
    .mock(method, path)
    .with_header("content-type", "application/json")
    .with_status(response_code)
    .with_body(response_body)
    .create();
    SwiftServerMock::new(server)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mullvad_api_mock_got_called(server: SwiftServerMock) -> bool {
    server.to_mock().matched()
}

use ios::*;
