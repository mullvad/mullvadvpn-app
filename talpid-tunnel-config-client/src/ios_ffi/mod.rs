pub mod ios_runtime;
pub mod ios_tcp_connection;

use crate::ios_ffi::ios_runtime::run_post_quantum_psk_exchange;
use libc::c_void;
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;

use std::sync::Once;
static INIT_LOGGING: Once = Once::new();

#[repr(C)]
pub struct PostQuantumCancelToken {
    // Must keep a pointer to a valid std::sync::Arc<tokio::mpsc::UnboundedSender>
    pub context: *mut c_void,
}

impl PostQuantumCancelToken {
    /// #Safety
    /// This function can only be called when the context pointer is valid.
    unsafe fn cancel(&self) {
        // Try to take the value, if there is a value, we can safely send the message, otherwise, assume it has been dropped and nothing happens
        let send_tx: Arc<mpsc::UnboundedSender<()>> = unsafe { Arc::from_raw(self.context as _) };
        let _ = send_tx.send(());
        std::mem::forget(send_tx);
    }
}

impl Drop for PostQuantumCancelToken {
    fn drop(&mut self) {
        let _: Arc<mpsc::UnboundedSender<()>> = unsafe { Arc::from_raw(self.context as _) };
    }
}
unsafe impl Send for PostQuantumCancelToken {}

#[no_mangle]
/**
 * # Safety
 * `sender` must be pointing to a valid instance of a `PostQuantumCancelToken` created by the `PacketTunnelProvider`
 */
pub unsafe extern "C" fn cancel_post_quantum_key_exchange(sender: *const PostQuantumCancelToken) {
    let sender = unsafe { &*sender };
    sender.cancel();
}

/**
 * # Safety
 * `sender` must be pointing to a valid instance of a `PostQuantumCancelToken` created by the `PacketTunnelProvider`.
 */
#[no_mangle]
pub unsafe extern "C" fn drop_post_quantum_key_exchange_token(
    sender: *const PostQuantumCancelToken,
) {
    let _sender = unsafe { std::ptr::read(sender) };
}

/**
 * # Safety
 * `sender` must be pointing to a valid instance of a `write_tx` created by the `IosTcpProvider`
 *
 * Callback to call when the TCP connection has written data.
 */
#[no_mangle]
pub unsafe extern "C" fn handle_sent(bytes_sent: usize, sender: *const c_void) {
    let weak_tx: Weak<mpsc::UnboundedSender<usize>> = unsafe { Weak::from_raw(sender as _) };
    if let Some(send_tx) = weak_tx.upgrade() {
        _ = send_tx.send(bytes_sent);
    }
}

/**
 * # Safety
 * `sender` must be pointing to a valid instance of a `read_tx` created by the `IosTcpProvider`
 *
 * Callback to call when the TCP connection has received data.
 */
#[no_mangle]
pub unsafe extern "C" fn handle_recv(data: *const u8, data_len: usize, sender: *const c_void) {
    let weak_tx: Weak<mpsc::UnboundedSender<Box<[u8]>>> = unsafe { Weak::from_raw(sender as _) };

    let mut bytes = vec![0u8; data_len];
    if !data.is_null() {
        std::ptr::copy_nonoverlapping(data, bytes.as_mut_ptr(), data_len);
    }
    if let Some(read_tx) = weak_tx.upgrade() {
        _ = read_tx.send(bytes.into_boxed_slice());
    }
}

/// Entry point for exchanging post quantum keys on iOS.
/// The TCP connection must be created to go through the tunnel.
/// # Safety
/// `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types.
/// They will not be valid after this function is called, and thus must be copied here.
/// `packet_tunnel` and `tcp_connection` must be valid pointers to a packet tunnel and a TCP connection
/// instances.
/// `cancel_token` should be owned by the caller of this function.
#[no_mangle]
pub unsafe extern "C" fn negotiate_post_quantum_key(
    public_key: *const u8,
    ephemeral_key: *const u8,
    packet_tunnel: *const c_void,
    tcp_connection: *const c_void,
    cancel_token: *mut PostQuantumCancelToken,
) -> i32 {
    INIT_LOGGING.call_once(|| {
        let _ = oslog::OsLogger::new("net.mullvad.MullvadVPN.TTCC")
            .level_filter(log::LevelFilter::Trace)
            .init();
    });

    let pub_key_copy: [u8; 32] = unsafe { std::ptr::read(public_key as *const [u8; 32]) };
    let eph_key_copy: [u8; 32] = unsafe { std::ptr::read(ephemeral_key as *const [u8; 32]) };

    match unsafe {
        run_post_quantum_psk_exchange(pub_key_copy, eph_key_copy, packet_tunnel, tcp_connection)
    } {
        Ok(token) => {
            unsafe { std::ptr::write(cancel_token, token) };
            0
        }
        Err(err) => err,
    }
}
