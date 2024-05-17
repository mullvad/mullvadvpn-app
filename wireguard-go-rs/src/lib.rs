//! This crate provides Rust bindings to wireguard-go with DAITA support.
//!
//! The bindings on the Go side are provided by `libwg`, which is a Go package that wraps
//! `wireguard-go` and provides a C FFI that we can use from Rust. On the Rust side, the FFI is
//! in the private `ffi` module below. It needs to be kept in sync with any changes to libwg.
//!
//! The [`Tunnel`] type provides a safe Rust wrapper around the C FFI.

#![cfg(unix)]

use core::slice;
use std::{
    ffi::{c_char, CStr},
    mem::ManuallyDrop,
};
use util::OnDrop;
use zeroize::Zeroize;

mod util;

pub type Fd = std::os::unix::io::RawFd;

pub type WgLogLevel = u32;

pub type LoggingContext = u64;
pub type LoggingCallback =
    unsafe extern "system" fn(level: WgLogLevel, msg: *const c_char, context: LoggingContext);

/// A wireguard-go tunnel
pub struct Tunnel {
    /// wireguard-go handle to the tunnel.
    handle: i32,
}

// NOTE: Must be kept in sync with libwg.go
// NOTE: must be kept in sync with `result_from_code`
// INVARIANT: Will always be represented as a negative i32
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum Error {
    #[error("Something went wrong.")]
    GeneralFailure = -1,

    #[error("Something went wrong, but trying again might help.")]
    IntermittentFailure = -2,

    #[error("An argument you provided was invalid.")]
    InvalidArgument = -3,

    #[error("The tunnel handle did not refer to an existing tunnel.")]
    UnknownTunnel = -4,

    #[error("The provided public key did not refer to an existing peer.")]
    UnknownPeer = -5,

    #[error("Something went wrong when enabling DAITA.")]
    EnableDaita = -6,

    #[error("`libwg` provided an unknown error code. This is a bug.")]
    Other = i32::MIN,
}

impl Tunnel {
    /// Creates a new wireguard tunnel, uses the specific interface name, and file descriptors
    /// for the tunnel device and logging. For targets other than android, this also takes an MTU value.
    ///
    /// The `logging_callback` let's you provide a Rust function that receives any logging output
    /// from wireguard-go. `logging_context` is a value that will be passed to each invocation of
    /// `logging_callback`.
    pub fn turn_on(
        #[cfg(not(target_os = "android"))] mtu: isize,
        settings: &CStr,
        device: Fd,
        logging_callback: Option<LoggingCallback>,
        logging_context: LoggingContext,
    ) -> Result<Self, Error> {
        // SAFETY: pointer is valid for the the lifetime of this function
        let code = unsafe {
            ffi::wgTurnOn(
                #[cfg(not(target_os = "android"))]
                mtu,
                settings.as_ptr(),
                device,
                logging_callback,
                logging_context,
            )
        };

        result_from_code(code)?;
        Ok(Tunnel { handle: code })
    }

    /// Stop the wireguard tunnel. This also happens automatically if the [`Tunnel`] is dropped.
    pub fn turn_off(self) -> Result<(), Error> {
        // we manually turn off the tunnel here, so wrap it in ManuallyDrop to prevent the Drop
        // impl from doing the same.
        let code = unsafe { ffi::wgTurnOff(self.handle) };
        let _ = ManuallyDrop::new(self);
        result_from_code(code)
    }

    /// Get the config of the WireGuard interface and make it available in the provided function.
    ///
    /// This takes a function to make sure the cstr get's zeroed and freed afterwards.
    /// Returns `None` if the call to wgGetConfig returned nil.
    ///
    /// **NOTE:** You should take extra care to avoid copying any secrets from the config without zeroizing them afterwards.
    // NOTE: this could return a guard type with a custom Drop impl instead, but me lazy.
    pub fn get_config<T>(&self, f: impl FnOnce(&CStr) -> T) -> Option<T> {
        let ptr = unsafe { ffi::wgGetConfig(self.handle) };

        if ptr.is_null() {
            return None;
        }

        // SAFETY: we checked for null, and wgGetConfig promises that this is a valid cstr
        let config = unsafe { CStr::from_ptr(ptr) };
        let config_len = config.to_bytes().len();

        // execute cleanup code on Drop to make sure that it happens even if `f` panics
        let on_drop = OnDrop::new(|| {
            {
                // SAFETY:
                // we checked for null, and wgGetConfig promises that this is a valid cstr.
                // config_len comes from the CStr above, so it should be good.
                let config_bytes = unsafe { slice::from_raw_parts_mut(ptr, config_len) };
                config_bytes.zeroize();
            }

            // SAFETY: the pointer was created by wgGetConfig, and we are no longer using it.
            unsafe { ffi::wgFreePtr(ptr.cast()) };
        });

        let t = f(config);
        let _ = config;
        drop(on_drop);

        Some(t)
    }

    /// Set the config of the WireGuard interface.
    pub fn set_config(&self, config: &CStr) -> Result<(), Error> {
        // SAFETY: pointer is valid for the lifetime of this function.
        let code = unsafe { ffi::wgSetConfig(self.handle, config.as_ptr()) };
        result_from_code(code)
    }

    /// Activate DAITA for the specified peer.
    ///
    /// `machines` is a string containing LF-separated maybenot machines.
    #[cfg(daita)]
    pub fn activate_daita(
        &self,
        peer_public_key: &[u8; 32],
        machines: &CStr,
        events_capacity: u32,
        actions_capacity: u32,
    ) -> Result<(), Error> {
        // SAFETY: pointers are valid for the lifetime of this function.
        let code = unsafe {
            ffi::wgActivateDaita(
                self.handle,
                peer_public_key.as_ptr(),
                machines.as_ptr(),
                events_capacity,
                actions_capacity,
            )
        };

        result_from_code(code)
    }

    /// Get the file descriptor of the tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    pub fn get_socket_v4(&self) -> Fd {
        unsafe { ffi::wgGetSocketV4(self.handle) }
    }

    /// Get the file descriptor of the tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    pub fn get_socket_v6(&self) -> Fd {
        unsafe { ffi::wgGetSocketV6(self.handle) }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        let code = unsafe { ffi::wgTurnOff(self.handle) };
        if let Err(e) = result_from_code(code) {
            log::error!("Failed to stop wireguard-go tunnel,oerror_code={code} ({e:?})")
        }
    }
}

fn result_from_code(code: i32) -> Result<(), Error> {
    // NOTE: must be kept in sync with enum definition
    Err(match code {
        0.. => return Ok(()),
        -1 => Error::GeneralFailure,
        -2 => Error::IntermittentFailure,
        -3 => Error::UnknownTunnel,
        -4 => Error::UnknownPeer,
        -5 => Error::EnableDaita,
        _ => Error::Other,
    })
}

impl Error {
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

mod ffi {
    use super::{Fd, LoggingCallback, LoggingContext};
    use core::ffi::{c_char, c_void};

    extern "C" {
        /// Creates a new wireguard tunnel, uses the specific interface name, and file descriptors
        /// for the tunnel device and logging. For targets other than android, this also takes an MTU value.
        ///
        /// Positive return values are tunnel handles for this specific wireguard tunnel instance.
        /// Negative return values signify errors.
        pub fn wgTurnOn(
            #[cfg(not(target_os = "android"))] mtu: isize,
            settings: *const c_char,
            fd: Fd,
            logging_callback: Option<LoggingCallback>,
            logging_context: LoggingContext,
        ) -> i32;

        /// Pass a handle that was created by wgTurnOn to stop a wireguard tunnel.
        ///
        /// Negative return values signify errors.
        pub fn wgTurnOff(handle: i32) -> i32;

        /// Get the config of the WireGuard interface. Returns null in case of error.
        ///
        /// # Safety:
        /// - The function returns an owned pointer to a null-terminated UTF-8 string.
        /// - The pointer may only be freed using [wgFreePtr].
        pub fn wgGetConfig(handle: i32) -> *mut c_char;

        /// Set the config of the WireGuard interface.
        ///
        /// Negative return values signify errors.
        ///
        /// # Safety:
        /// - `settings` must point to a null-terminated UTF-8 string.
        /// - The pointer will not be read from after `wgActivateDaita` has returned.
        pub fn wgSetConfig(handle: i32, settings: *const c_char) -> i32;

        /// Activate DAITA for the specified peer.
        ///
        /// `tunnel_handle` must come from [wgTurnOn]. `machines` is a string containing LF-separated
        /// maybenot machines.
        ///
        /// Negative return values signify errors.
        ///
        /// # Safety:
        /// - `peer_public_key` must point to a 32 byte array.
        /// - `machines` must point to a null-terminated UTF-8 string.
        /// - Neither pointer will be read from after `wgActivateDaita` has returned.
        #[cfg(daita)]
        pub fn wgActivateDaita(
            tunnel_handle: i32,
            peer_public_key: *const u8,
            machines: *const c_char,
            events_capacity: u32,
            actions_capacity: u32,
        ) -> i32;

        /// Free a pointer allocated by the go runtime - useful to free return value of wgGetConfig
        pub fn wgFreePtr(ptr: *mut c_void);

        /// Get the file descriptor of the tunnel IPv4 socket.
        #[cfg(target_os = "android")]
        pub fn wgGetSocketV4(handle: i32) -> Fd;

        /// Get the file descriptor of the tunnel IPv6 socket.
        #[cfg(target_os = "android")]
        pub fn wgGetSocketV6(handle: i32) -> Fd;
    }
}
