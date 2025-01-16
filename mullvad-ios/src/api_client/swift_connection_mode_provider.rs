use std::{
    ffi::{c_char, c_void, CStr},
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    os::unix,
    ptr::null_mut,
    sync::Arc,
    time::Duration,
};

use futures::{SinkExt, StreamExt};

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use mullvad_api::proxy::{ApiConnectionMode, ConnectionModeProvider, ProxyConfig};

use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState as State;
use talpid_types::net::proxy;

use crate::{
    encrypted_dns_proxy::EncryptedDnsProxyState, ios::mullvad_ios_runtime, tunnel_obfuscator_proxy,
};
extern "C" {
    pub fn connection_mode_provider_initial(rawPointer: *const c_void);
    pub fn connection_mode_provider_rotate(rawPointer: *const c_void);
    pub fn connection_mode_provider_receive(rawIterator: *const c_void) -> *const c_void;
}

unsafe fn convert_c_string(c_str: *const c_char) -> String {
    // SAFETY: domain_name points to a valid region of memory and contains a null terminator.
    let str = unsafe { CStr::from_ptr(c_str) };
    return String::from_utf8_lossy(str.to_bytes()).into_owned();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_connection_mode_provider(
    raw_provider: *const c_void,
    domain_name: *const c_char,
) -> SwiftConnectionModeProvider {
    let domain = {
        // SAFETY: domain_name points to a valid region of memory and contains a null terminator.
        let c_str = unsafe { CStr::from_ptr(domain_name) };
        String::from_utf8_lossy(c_str.to_bytes())
    };

    let state = EncryptedDnsProxyState {
        state: State::default(),
        domain: domain.into_owned(),
    };

    let (receive_tx, receive_rx) = mpsc::unbounded();

    let context = SwiftConnectionModeProviderContext {
        provider: raw_provider,
        encrypted_dns_state: state,
        receive_tx,
        receive_rx,
    };

    SwiftConnectionModeProvider::new(context)
}

// TODO: Use 1 FFI function per enum parameter to expose a functional interface

#[repr(C)]
pub struct SwiftConnectionModeProvider(*mut SwiftConnectionModeProviderContext);
impl SwiftConnectionModeProvider {
    pub fn new(context: SwiftConnectionModeProviderContext) -> SwiftConnectionModeProvider {
        SwiftConnectionModeProvider(Box::into_raw(Box::new(context)))
    }

    pub unsafe fn into_rust_context(self) -> Box<SwiftConnectionModeProviderContext> {
        Box::from_raw(self.0)
    }
}

#[no_mangle]
pub unsafe extern "C" fn convert_direct() -> *const c_void {
    Box::into_raw(Box::new(ApiConnectionMode::Direct)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn convert_shadowsocks(
    address: *const u8,
    address_len: usize,
    port: u16,
    c_password: *const c_char,
    c_cipher: *const c_char,
) -> *const c_void {
    let endpoint: SocketAddr = if let Some(ip_address) = parse_ip_addr(address, address_len) {
        SocketAddr::new(ip_address, port)
    } else {
        return std::ptr::null();
    };

    let password = convert_c_string(c_password);
    let cipher = convert_c_string(c_cipher);

    let shadowsocks_configuration = ProxyConfig::Shadowsocks(proxy::Shadowsocks {
        endpoint,
        password,
        cipher,
    });

    let connection_mode = ApiConnectionMode::Proxied(shadowsocks_configuration);

    return Box::into_raw(Box::new(connection_mode)) as *mut c_void;
}

pub struct SwiftConnectionModeProviderContext {
    provider: *const c_void,
    encrypted_dns_state: EncryptedDnsProxyState,
    receive_tx: mpsc::UnboundedSender<()>,
    receive_rx: mpsc::UnboundedReceiver<()>,
}

impl SwiftConnectionModeProviderContext {
    pub fn spawn_rotator(&self) -> impl Future<Output = ()> {
        let mut tx = self.receive_tx.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let _ = tx.send(()).await;
            }
        }
    }
}
unsafe impl Send for SwiftConnectionModeProviderContext {}

impl ConnectionModeProvider for SwiftConnectionModeProviderContext {
    fn initial(&self) -> ApiConnectionMode {
        unsafe {
            connection_mode_provider_initial(self.provider);
        }
        ApiConnectionMode::Direct
    }

    fn rotate(&self) -> impl std::future::Future<Output = ()> + Send {
        unsafe {
            connection_mode_provider_rotate(self.provider);
        }
        _ = self.receive_tx.unbounded_send(());
        futures::future::ready(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Option<ApiConnectionMode>> + Send {
        // let runtime = mullvad_ios_runtime().unwrap();
        // runtime.spawn_blocking(func)

        let raw_method = unsafe { connection_mode_provider_receive(self.provider) };

        let method: ApiConnectionMode = unsafe { *Box::from_raw(raw_method as *mut _) };

        return async {
            if let Some(_) = self.receive_rx.next().await {
                Some(method)
            } else {
                None
            }
        };
    }
}

/// Constructs a new IP address from a pointer containing bytes representing an IP address.
///
/// SAFETY: `addr` must be a pointer to at least `addr_len` bytes.
pub unsafe fn parse_ip_addr(addr: *const u8, addr_len: usize) -> Option<IpAddr> {
    match addr_len {
        4 => {
            // SAFETY: addr pointer must point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).into())
        }
        16 => {
            // SAFETY: addr pointer must point to at least addr_len bytes
            let bytes = unsafe { std::slice::from_raw_parts(addr, addr_len) };
            let mut addr_arr = [0u8; 16];
            addr_arr.as_mut_slice().copy_from_slice(bytes);

            Some(Ipv6Addr::from(addr_arr).into())
        }
        anything_else => {
            log::error!("Bad IP address length {anything_else}");
            None
        }
    }
}
