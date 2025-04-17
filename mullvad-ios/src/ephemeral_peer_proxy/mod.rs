#![cfg(target_os = "ios")]
pub mod ios_tcp_connection;
pub mod peer_exchange;

use libc::c_void;
use peer_exchange::EphemeralPeerExchange;

use std::{ffi::CString, ptr, sync::Once};
use talpid_tunnel_config_client::DaitaSettings;
static INIT_LOGGING: Once = Once::new();

#[derive(Clone)]
pub struct PacketTunnelBridge {
    pub packet_tunnel: *const c_void,
    pub tunnel_handle: i32,
}

impl PacketTunnelBridge {
    fn fail_exchange(self) {
        // # Safety:
        // Call is safe as long as the `packet_tunnel` pointer is valid. Since a valid instance of
        // `PacketTunnelBridge` requires the packet tunnel pointer to be valid, it is assumed this
        // call is safe.
        unsafe {
            swift_ephemeral_peer_ready(self.packet_tunnel, ptr::null(), ptr::null(), ptr::null())
        };
    }

    fn succeed_exchange(
        self,
        ephemeral_key: [u8; 32],
        preshared_key: Option<[u8; 32]>,
        daita: Option<DaitaParameters>,
    ) {
        let ephemeral_ptr = ephemeral_key.as_ptr();
        let preshared_ptr = preshared_key
            .as_ref()
            .map(|key| key.as_ptr())
            .unwrap_or(ptr::null());

        let daita_ptr = daita
            .as_ref()
            .map(|params| params as *const _)
            .unwrap_or(ptr::null());
        // # Safety:
        // The `packet_tunnel` pointer must be valid, much like the call in `fail_exchange`, but
        // since the other arguments here are non-null, these pointers (`preshared_ptr`,
        // `ephmerela_ptr` and `daita_ptr`) have to be valid too. Since they point to local
        // variables or are null, the pointer values will be valid for the lifetime of the call.
        unsafe {
            swift_ephemeral_peer_ready(self.packet_tunnel, preshared_ptr, ephemeral_ptr, daita_ptr)
        };
    }
}

// SAFETY: See notes for `EphemeralPeerExchange`
unsafe impl Send for PacketTunnelBridge {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EphemeralPeerParameters {
    pub peer_exchange_timeout: u64,
    pub enable_post_quantum: bool,
    pub enable_daita: bool,
    pub funcs: ios_tcp_connection::WgTcpConnectionFunctions,
}

#[repr(C)]
pub struct DaitaParameters {
    pub machines: *mut u8,
    pub max_padding_frac: f64,
    pub max_blocking_frac: f64,
}

impl DaitaParameters {
    fn new(settings: DaitaSettings) -> Option<Self> {
        let machines_string = settings.client_machines.join("\n");
        let machines = CString::new(machines_string).ok()?.into_raw().cast();
        Some(Self {
            machines,
            max_padding_frac: settings.max_padding_frac,
            max_blocking_frac: settings.max_blocking_frac,
        })
    }
}

impl Drop for DaitaParameters {
    fn drop(&mut self) {
        // # Safety:
        // `machines` pointer must be a valid pointer to a CString. This can be achieved by
        // ensuring that `DaitaParameters` are constructed via `DaitaParameters::new` and the
        // `machines` pointer is never written to.
        let _ = unsafe { CString::from_raw(self.machines.cast()) };
    }
}

extern "C" {
    /// To be called when ephemeral peer exchange has finished. All parameters except
    /// `raw_packet_tunnel` are optional.
    ///
    /// # Safety:
    /// If the key exchange failed, all pointers except `raw_packet_tunnel` must be null. If the
    /// key exchange was successful, `raw_ephemeral_private_key` must be a valid pointer to 32
    /// bytes for the lifetime of this call. If PQ was enabled, `raw_preshared_key` must be a valid
    /// pointer to 32 bytes for the lifetime of this call. If DAITA was requested, the
    /// `daita_prameters` must point to a valid instance of `DaitaParameters`.
    pub fn swift_ephemeral_peer_ready(
        raw_packet_tunnel: *const c_void,
        raw_preshared_key: *const u8,
        raw_ephemeral_private_key: *const u8,
        daita_parameters: *const DaitaParameters,
    );

}

/// Called by the Swift side to signal that the ephemeral peer exchange should be cancelled.
/// After this call, the cancel token is no longer valid.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cancel_ephemeral_peer_exchange(
    sender: *mut peer_exchange::ExchangeCancelToken,
) {
    // SAFETY: See notes above
    let sender = unsafe { Box::from_raw(sender) };
    sender.cancel();
}

/// Called by the Swift side to signal that the Rust `EphemeralPeerCancelToken` can be safely
/// dropped from memory.
///
/// # Safety
/// `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
/// `PacketTunnelProvider`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn drop_ephemeral_peer_exchange_token(
    sender: *mut peer_exchange::ExchangeCancelToken,
) {
    // SAFETY: See notes above
    // drop the cancel token
    let _sender = unsafe { Box::from_raw(sender) };
}

/// Entry point for requesting ephemeral peers on iOS.
/// The TCP connection must be created to go through the tunnel.
/// # Safety
/// `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types,
/// specifically, they must be valid pointers to 32 bytes. They will not be valid after this
/// function is called, and thus must be copied here. `packet_tunnel` must be valid pointers to a
/// packet tunnel, the packet tunnel pointer must outlive the ephemeral peer exchange.
/// `cancel_token` should be owned by the caller of this function.
#[unsafe(no_mangle)]
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

    // # Safety:
    // `public_key` pointer must be a valid pointer to 32 unsigned bytes.
    let pub_key: [u8; 32] = unsafe { ptr::read(public_key as *const [u8; 32]) };
    // # Safety:
    // `ephemeral_key` pointer must be a valid pointer to 32 unsigned bytes.
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
