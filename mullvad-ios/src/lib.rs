#![cfg(target_os = "ios")]
mod encrypted_dns_proxy;
mod ephemeral_peer_proxy;
mod shadowsocks_proxy;
mod api;
pub mod tunnel_obfuscator_proxy;

#[repr(C)]
pub struct ProxyHandle {
    pub context: *mut std::ffi::c_void,
    pub port: u16,
}

#[no_mangle]
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

use ios::*;
use mullvad_api::{
    proxy::{ApiConnectionMode, StaticConnectionModeProvider},
    rest::{MullvadRestHandle, Response},
    ApiEndpoint, ApiProxy, Runtime,
};
use std::{
    ffi::{CStr, CString},
    net::Incoming,
    ptr::{null, null_mut},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
    u8,
};
use tokio::task::JoinHandle;


