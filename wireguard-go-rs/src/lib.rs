#![cfg(unix)]
pub type Fd = std::os::unix::io::RawFd;
use std::ffi::{c_char, c_void};
pub type WgLogLevel = u32;
pub type LoggingCallback =
    unsafe extern "system" fn(level: WgLogLevel, msg: *const c_char, context: *mut c_void);

extern "C" {
    /// Creates a new wireguard tunnel, uses the specific interface name, MTU and file descriptors
    /// for the tunnel device and logging.
    ///
    /// Positive return values are tunnel handles for this specific wireguard tunnel instance.
    /// Negative return values signify errors. All error codes are opaque.
    #[cfg(not(target_os = "android"))]
    pub fn wgTurnOn(
        mtu: isize,
        settings: *const i8,
        fd: Fd,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut c_void,
    ) -> i32;

    // Android
    #[cfg(target_os = "android")]
    pub fn wgTurnOn(
        settings: *const i8,
        fd: Fd,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut c_void,
    ) -> i32;

    // Pass a handle that was created by wgTurnOn to stop a wireguard tunnel.
    pub fn wgTurnOff(handle: i32) -> i32;

    // Returns the file descriptor of the tunnel IPv4 socket.
    pub fn wgGetConfig(handle: i32) -> *mut c_char;

    // Sets the config of the WireGuard interface.
    pub fn wgSetConfig(handle: i32, settings: *const i8) -> i32;

    /// Activate DAITA for the specified peer.
    ///
    /// `tunnel_handle` must come from [wgTurnOn]. `machines` is a string containing LF-separated
    /// maybenot machines.
    ///
    /// # Safety:
    /// - `peer_public_key` must point to a 32 byte array.
    /// - `machines` must point to a null-terminated UTF-8 string.
    /// - Neither pointer will be written to by `wgActivateDaita`.
    /// - Neither pointer will be read from after `wgActivateDaita` has returned.
    #[cfg(daita)]
    pub fn wgActivateDaita(
        tunnel_handle: i32,
        peer_public_key: *const u8,
        machines: *const c_char,
        events_capacity: u32,
        actions_capacity: u32,
    ) -> i32;

    // Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    pub fn wgFreePtr(ptr: *mut c_void);

    // Returns the file descriptor of the tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    pub fn wgGetSocketV4(handle: i32) -> Fd;

    // Returns the file descriptor of the tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    pub fn wgGetSocketV6(handle: i32) -> Fd;
}
