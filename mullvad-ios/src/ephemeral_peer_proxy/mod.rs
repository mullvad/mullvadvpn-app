#![cfg(target_os = "ios")]
pub mod peer_exchange;
pub mod ios_tcp_connection;

use peer_exchange::EphemeralPeerExchange;
use libc::c_void;

use std::sync::Once;
static INIT_LOGGING: Once = Once::new();

#[derive(Clone)]
pub struct PacketTunnelBridge {
    pub packet_tunnel: *const c_void,
    pub tunnel_handle: i32,
}

// # Safety
// This is safe as long as the PacketTunnel class outlives the instance of the PacketTunnelBridge,
// and thus the ephemeral peer exchange. Since the peer exchange takes place in the packet tunnel
// process on iOS, it is certain _enough_ this will be the case.
// It is safe to implement Send for PacketTunnelBridge because the packet_tunnel
unsafe impl Send for PacketTunnelBridge {}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct EphemeralPeerParameters {
    pub peer_exchange_timeout: u64,
    pub enable_post_quantum: bool,
    pub enable_daita: bool,
    pub funcs: ios_tcp_connection::WgTcpConnectionFuncs,
}

#[repr(C)]
pub struct EphemeralPeerCancelToken {
    cancel_token: *mut peer_exchange::ExchangeCancelToken,
}

impl EphemeralPeerCancelToken {
    fn new(cancel_token: peer_exchange::ExchangeCancelToken) -> Self {
        let ptr = Box::into_raw(Box::new(cancel_token));
        Self { cancel_token: ptr }
    }

    /// # Safety
    /// This function can only be called when the context pointer is valid.
    unsafe fn cancel(&self) {
        let token = unsafe { std::ptr::read(self.cancel_token) };
        token.cancel();

        std::mem::forget(token);
    }

    fn to_ptr(self) -> *mut Self {
        let boxed = Box::new(self);
        Box::into_raw(boxed)
    }
}

impl Drop for EphemeralPeerCancelToken {
    fn drop(&mut self) {
        let _token = unsafe { Box::from_raw(self.cancel_token) };
    }
}

/// Called by the Swift side to signal that the ephemeral peer exchange should be cancelled.
/// After this call, the cancel token is no longer valid.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[no_mangle]
pub unsafe extern "C" fn cancel_ephemeral_peer_exchange(sender: *mut EphemeralPeerCancelToken) {
    let sender = unsafe { Box::from_raw(sender) };
    sender.cancel();
}

/// Called by the Swift side to signal that the Rust `EphemeralPeerCancelToken` can be safely dropped
/// from memory.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[no_mangle]
pub unsafe extern "C" fn drop_ephemeral_peer_exchange_token(
    sender: *mut EphemeralPeerCancelToken,
) {

    // drop the cancel token
    let _sender = unsafe { Box::from_raw(sender) };
}

/// Entry point for requesting ephemeral peers on iOS.
/// The TCP connection must be created to go through the tunnel.
/// # Safety
/// `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types.
/// They will not be valid after this function is called, and thus must be copied here.
/// `packet_tunnel` and `tcp_connection` must be valid pointers to a packet tunnel and a TCP
/// connection instances.
/// `cancel_token` should be owned by the caller of this function.
#[no_mangle]
pub unsafe extern "C" fn request_ephemeral_peer(
    public_key: *const u8,
    ephemeral_key: *const u8,
    packet_tunnel: *const c_void,
    tunnel_handle: i32,
    peer_parameters: EphemeralPeerParameters,
) -> *mut EphemeralPeerCancelToken {
    INIT_LOGGING.call_once(|| {
        let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.TTCC")
            .level_filter(log::LevelFilter::Debug)
            .init();
    });

    let pub_key: [u8; 32] = unsafe { std::ptr::read(public_key as *const [u8; 32]) };
    let eph_key: [u8; 32] = unsafe { std::ptr::read(ephemeral_key as *const [u8; 32]) };

    let handle = match crate::mullvad_ios_runtime() {
        Ok(handle) => handle,
        Err(err) => {
            log::error!("Failed to obtain a handle to a tokio runtime: {err}");

            return std::ptr::null_mut();
        }
    };

    let packet_tunnel_bridge = PacketTunnelBridge {
        packet_tunnel,
        tunnel_handle,
    };


    let cancel_token =
        EphemeralPeerExchange::new(pub_key, eph_key, packet_tunnel_bridge, peer_parameters)
            .run(handle);

    EphemeralPeerCancelToken::new(cancel_token).to_ptr()
}
