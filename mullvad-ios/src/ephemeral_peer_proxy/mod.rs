#![cfg(target_os = "ios")]
pub mod ios_runtime;
pub mod ios_tcp_connection;

use ios_runtime::run_ephemeral_peer_exchange;
use ios_tcp_connection::ConnectionContext;
use libc::c_void;
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::mpsc;

use std::sync::Once;
static INIT_LOGGING: Once = Once::new();

#[repr(C)]
pub struct EphemeralPeerCancelToken {
    // Must keep a pointer to a valid std::sync::Arc<tokio::mpsc::UnboundedSender>
    pub context: *mut c_void,
}

pub struct PacketTunnelBridge {
    pub packet_tunnel: *const c_void,
    pub tcp_connection: *const c_void,
}

pub struct EphemeralPeerParameters {
    pub peer_exchange_timeout: u64,
    pub enable_post_quantum: bool,
    pub enable_daita: bool,
}

impl EphemeralPeerCancelToken {
    /// # Safety
    /// This function can only be called when the context pointer is valid.
    unsafe fn cancel(&self) {
        // # Safety
        // Try to take the value, if there is a value, we can safely send the message, otherwise,
        // assume it has been dropped and nothing happens
        let connection_context: Arc<Mutex<ConnectionContext>> =
            unsafe { Arc::from_raw(self.context as _) };
        if let Ok(mut connection) = connection_context.lock() {
            connection.shutdown();
        }

        // Call std::mem::forget here to avoid dropping the channel.
        std::mem::forget(connection_context);
    }
}

impl Drop for EphemeralPeerCancelToken {
    fn drop(&mut self) {
        let _: Arc<Mutex<ConnectionContext>> = unsafe { Arc::from_raw(self.context as _) };
    }
}

unsafe impl Send for EphemeralPeerCancelToken {}

/// Called by the Swift side to signal that the ephemeral peer exchange should be cancelled.
/// After this call, the cancel token is no longer valid.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[no_mangle]
pub unsafe extern "C" fn cancel_ephemeral_peer_exchange(sender: *const EphemeralPeerCancelToken) {
    let sender = unsafe { &*sender };
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
    sender: *const EphemeralPeerCancelToken,
) {
    let _sender = unsafe { std::ptr::read(sender) };
}

/// Called by Swift whenever data has been written to the in-tunnel TCP connection when exchanging
/// quantum-resistant pre shared keys, or ephemeral peers.
///
/// If `bytes_sent` is 0, this indicates that the connection was closed or that an error occurred.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `write_tx` created by the `IosTcpProvider`
/// Callback to call when the TCP connection has written data.
#[no_mangle]
pub unsafe extern "C" fn handle_sent(bytes_sent: usize, sender: *const c_void) {
    let weak_tx: Weak<mpsc::UnboundedSender<usize>> = unsafe { Weak::from_raw(sender as _) };
    if let Some(send_tx) = weak_tx.upgrade() {
        _ = send_tx.send(bytes_sent);
    }
}

/// Called by Swift whenever data has been read from the in-tunnel TCP connection when exchanging
/// quantum-resistant pre shared keys, or ephemeral peers.
///
/// If `data` is null or empty, this indicates that the connection was closed or that an error
/// occurred. An empty buffer is sent to the underlying reader to signal EOF.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `read_tx` created by the `IosTcpProvider`
///
/// Callback to call when the TCP connection has received data.
#[no_mangle]
pub unsafe extern "C" fn handle_recv(data: *const u8, mut data_len: usize, sender: *const c_void) {
    let weak_tx: Weak<mpsc::UnboundedSender<Box<[u8]>>> = unsafe { Weak::from_raw(sender as _) };

    if data.is_null() {
        data_len = 0;
    }
    let mut bytes = vec![0u8; data_len];
    if !data.is_null() {
        std::ptr::copy_nonoverlapping(data, bytes.as_mut_ptr(), data_len);
    }
    if let Some(read_tx) = weak_tx.upgrade() {
        _ = read_tx.send(bytes.into_boxed_slice());
    }
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
    tcp_connection: *const c_void,
    cancel_token: *mut EphemeralPeerCancelToken,
    peer_exchange_timeout: u64,
    enable_post_quantum: bool,
    enable_daita: bool,
) -> i32 {
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

            return -1;
        }
    };

    let packet_tunnel_bridge = PacketTunnelBridge {
        packet_tunnel,
        tcp_connection,
    };
    let peer_parameters = EphemeralPeerParameters {
        peer_exchange_timeout,
        enable_post_quantum,
        enable_daita,
    };

    match unsafe {
        run_ephemeral_peer_exchange(
            pub_key,
            eph_key,
            packet_tunnel_bridge,
            peer_parameters,
            handle,
        )
    } {
        Ok(token) => {
            unsafe { std::ptr::write(cancel_token, token) };
            0
        }
        Err(_) => -1,
    }
}
