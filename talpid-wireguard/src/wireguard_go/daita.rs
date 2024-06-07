use std::{ffi::CStr, io};

use talpid_types::net::wireguard::PublicKey;
use wireguard_go_rs::wgActivateDaita;

/// Maximum number of events that can be stored in the underlying buffer
const EVENTS_CAPACITY: u32 = 1000;
/// Maximum number of actions that can be stored in the underlying buffer
const ACTIONS_CAPACITY: u32 = 1000;

#[derive(Debug)]
pub struct Session {
    _tunnel_handle: i32,
}

impl Session {
    /// Enable DAITA for an existing WireGuard interface.
    pub(super) fn from_adapter(
        tunnel_handle: i32,
        peer_public_key: &PublicKey,
        machines: &CStr,
    ) -> io::Result<Session> {
        // SAFETY:
        // peer_public_key and machines lives for the duration of this function call.

        // TODO: ´machines` must be valid UTF-8
        let res = unsafe {
            wgActivateDaita(
                tunnel_handle,
                peer_public_key.as_bytes().as_ptr(),
                machines.as_ptr(),
                EVENTS_CAPACITY,
                ACTIONS_CAPACITY,
            )
        };
        if res < 0 {
            // TODO: return error
            panic!("Failed to activate DAITA on tunnel {tunnel_handle}")
        }
        Ok(Self {
            _tunnel_handle: tunnel_handle,
        })
    }
}
