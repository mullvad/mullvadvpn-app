use libc::c_void;
use tokio::sync::mpsc;

use crate::PostQuantumCancelToken;

use super::run_ios_runtime;

use std::sync::Once;
static INIT_LOGGING: Once = Once::new();

#[no_mangle]
pub unsafe extern "C" fn cancel_post_quantum_key_exchange(sender: *const PostQuantumCancelToken) {
    let sender = unsafe { &*sender };
    sender.cancel();
}

#[no_mangle]
pub unsafe extern "C" fn drop_post_quantum_key_exchange_token(
    sender: *const PostQuantumCancelToken,
) {
    let _sender = unsafe { std::ptr::read(sender) };
}

/// Callback to call when the TCP connection has written data.
#[no_mangle]
pub unsafe extern "C" fn handle_sent(bytes_sent: usize, sender: *const c_void) {
    let send_tx: Box<mpsc::UnboundedSender<usize>> = unsafe { Box::from_raw(sender as _) };
    _ = send_tx.send(bytes_sent);
}

/// Callback to call when the TCP connection has received data.
#[no_mangle]
pub unsafe extern "C" fn handle_recv(data: *const u8, data_len: usize, sender: *const c_void) {
    let read_tx: Box<mpsc::UnboundedSender<Box<[u8]>>> = unsafe { Box::from_raw(sender as _) };
    let mut bytes = vec![0u8; data_len];
    if !data.is_null() {
        std::ptr::copy_nonoverlapping(data, bytes.as_mut_ptr(), data_len);
    }
    _ = read_tx.send(bytes.into_boxed_slice());
}

/// Entry point for exchanging post quantum keys on iOS.
/// The TCP connection must be created to go through the tunnel.
/// # Safety
/// This function is safe to call
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

    match unsafe { run_ios_runtime(pub_key_copy, eph_key_copy, packet_tunnel, tcp_connection) } {
        Ok(token) => {
            unsafe { std::ptr::write(cancel_token, token) };
            0
        }
        Err(err) => err,
    }
}
