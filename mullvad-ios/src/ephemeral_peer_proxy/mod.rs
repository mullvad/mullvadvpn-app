#![cfg(target_os = "ios")]
pub mod ios_tcp_connection;
pub mod peer_exchange;

use ios_tcp_connection::swift_ephemeral_peer_ready;
use libc::c_void;
use peer_exchange::EphemeralPeerExchange;

use std::{ptr, sync::Once};
static INIT_LOGGING: Once = Once::new();

#[derive(Clone)]
pub struct PacketTunnelBridge {
    pub packet_tunnel: *const c_void,
    pub tunnel_handle: i32,
}

impl PacketTunnelBridge {
    fn fail_exchange(self) {
        unsafe { swift_ephemeral_peer_ready(self.packet_tunnel, ptr::null(), ptr::null()) };
    }

    fn succeed_exchange(self, ephemeral_key: [u8; 32], preshared_key: Option<[u8; 32]>) {
        let ephemeral_ptr = ephemeral_key.as_ptr();
        let preshared_ptr = preshared_key
            .as_ref()
            .map(|key| key.as_ptr())
            .unwrap_or(ptr::null());

        unsafe { swift_ephemeral_peer_ready(self.packet_tunnel, preshared_ptr, ephemeral_ptr) };
    }
}

unsafe impl Send for PacketTunnelBridge {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EphemeralPeerParameters {
    pub peer_exchange_timeout: u64,
    pub enable_post_quantum: bool,
    pub enable_daita: bool,
    pub funcs: ios_tcp_connection::WgTcpConnectionFunctions,
}

/// Called by the Swift side to signal that the ephemeral peer exchange should be cancelled.
/// After this call, the cancel token is no longer valid.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[no_mangle]
pub unsafe extern "C" fn cancel_ephemeral_peer_exchange(
    sender: *mut peer_exchange::ExchangeCancelToken,
) {
    let sender = unsafe { Box::from_raw(sender) };
    sender.cancel();
}

/// Called by the Swift side to signal that the Rust `EphemeralPeerCancelToken` can be safely
/// dropped from memory.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[no_mangle]
pub unsafe extern "C" fn drop_ephemeral_peer_exchange_token(
    sender: *mut peer_exchange::ExchangeCancelToken,
) {
    // drop the cancel token
    let _sender = unsafe { Box::from_raw(sender) };
}

/// Entry point for requesting ephemeral peers on iOS.
/// The TCP connection must be created to go through the tunnel.
/// # Safety
/// `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types.
/// They will not be valid after this function is called, and thus must be copied here.
/// `packet_tunnel` must be valid pointers to a packet tunnel, the packet tunnel pointer must
/// outlive the ephemeral peer exchange. `cancel_token` should be owned by the caller of this
/// function.
#[no_mangle]
pub unsafe extern "C" fn request_ephemeral_peer(
    public_key: *const u8,
    ephemeral_key: *const u8,
    packet_tunnel: *const c_void,
    tunnel_handle: i32,
    peer_parameters: EphemeralPeerParameters,
) -> *mut peer_exchange::ExchangeCancelToken {
    INIT_LOGGING.call_once(|| {
        let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.TTCC")
            .level_filter(log::LevelFilter::Debug)
            .init();
    });

    let pub_key: [u8; 32] = unsafe { ptr::read(public_key as *const [u8; 32]) };
    let eph_key: [u8; 32] = unsafe { ptr::read(ephemeral_key as *const [u8; 32]) };

    let handle = match crate::mullvad_ios_runtime() {
        Ok(handle) => handle,
        Err(err) => {
            log::error!("Failed to obtain a handle to a tokio runtime: {err}");

            return ptr::null_mut();
        }
    };

    let packet_tunnel_bridge = PacketTunnelBridge {
        packet_tunnel,
        tunnel_handle,
    };

    let cancel_token =
        EphemeralPeerExchange::new(pub_key, eph_key, packet_tunnel_bridge, peer_parameters)
            .run(handle);

    Box::into_raw(Box::new(cancel_token))
}
