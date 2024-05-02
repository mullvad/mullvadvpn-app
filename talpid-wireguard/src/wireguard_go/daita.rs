use std::{ffi::CStr, io};

/// Maximum number of events that can be stored in the underlying buffer
const EVENTS_CAPACITY: u32 = 1000;
/// Maximum number of actions that can be stored in the underlying buffer
const ACTIONS_CAPACITY: u32 = 1000;

#[derive(Debug)]
pub struct Session {
    _tunnel_handle: i32,
}

impl Session {
    /// Call `wgActivateDaita` for an existing WireGuard interface
    pub(super) fn from_adapter(tunnel_handle: i32, machines: &CStr) -> io::Result<Session> {
        let res = unsafe {
            super::wgActivateDaita(
                machines.as_ptr(),
                tunnel_handle,
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

    // TODO:
    // pub(super) fn stop(self) { ... }
}
