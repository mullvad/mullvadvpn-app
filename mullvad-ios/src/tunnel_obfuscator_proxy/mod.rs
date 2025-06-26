use libc::c_char;
use std::{
    ffi::CStr, io, net::{Ipv4Addr, SocketAddr}
};
use tokio::task::JoinHandle;
use tunnel_obfuscation::{
    create_obfuscator, quic, shadowsocks, udp2tcp, Settings as ObfuscationSettings,
};

mod ffi;

use crate::mullvad_ios_runtime;

pub struct TunnelObfuscatorRuntime {
    settings: ObfuscationSettings,
}

impl TunnelObfuscatorRuntime {
    pub fn new_udp2tcp(peer: SocketAddr) -> Self {
        let settings = ObfuscationSettings::Udp2Tcp(udp2tcp::Settings { peer });
        Self { settings }
    }

    pub fn new_shadowsocks(peer: SocketAddr) -> Self {
        let settings = ObfuscationSettings::Shadowsocks(shadowsocks::Settings {
            shadowsocks_endpoint: peer,
            wireguard_endpoint: SocketAddr::from((Ipv4Addr::LOCALHOST, 51820)),
        });
        Self { settings }
    }

    pub fn new_quic(peer: SocketAddr, hostname: String, token: String) -> Self {
        let settings = ObfuscationSettings::Quic(quic::Settings {
            quic_endpoint: peer,
            wireguard_endpoint: SocketAddr::from((Ipv4Addr::LOCALHOST, 51820)),
            hostname,
            token,
        });
        Self { settings }
    }

    pub fn run(self) -> io::Result<(SocketAddr, TunnelObfuscatorHandle)> {
        let runtime = mullvad_ios_runtime().map_err(io::Error::other)?;

        let obfuscator = runtime.block_on(async move {
            create_obfuscator(&self.settings)
                .await
                .map_err(io::Error::other)
        })?;

        let endpoint = obfuscator.endpoint();
        let join_handle = runtime.spawn(async move {
            let _ = obfuscator.run().await;
        });

        Ok((
            endpoint,
            TunnelObfuscatorHandle {
                obfuscator_abort_handle: join_handle,
            },
        ))
    }
}

pub struct TunnelObfuscatorHandle {
    obfuscator_abort_handle: JoinHandle<()>,
}

impl TunnelObfuscatorHandle {
    pub fn stop(self) {
        self.obfuscator_abort_handle.abort();
    }
}

/// Try to convert a C string to an owned [String]. if `ptr` is null, an empty [String] is
/// returned.
///
/// # Safety
/// - `ptr` must uphold all safety invariants as required by [CStr::from_ptr].
fn get_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // Safety: See function doc comment.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().map(ToOwned::to_owned).unwrap_or_default()
}
