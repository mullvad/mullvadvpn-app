//! C FFI for the iOS tunnel adapter.
//!
//! Config is built using the builder pattern:
//! 1. `gotatun_config_new(private_key, ipv4_addr, exit_public_key, exit_endpoint)` → config handle
//! 2. `gotatun_config_set_*()` — set additional params
//! 3. `gotatun_start_tunnel(tun_fd, config, callbacks)` → tunnel handle
//! 4. `gotatun_stop_tunnel(handle)` — stop and free

mod config;
mod lifecycle;

use std::{ffi::CStr, os::raw::c_char};

fn parse_cstr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        log::error!("gotatun FFI: received null C string pointer");
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}

fn read_32_bytes(ptr: *const u8) -> Option<[u8; 32]> {
    if ptr.is_null() {
        log::error!("gotatun FFI: received null key pointer");
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, 32) };
    let mut arr = [0u8; 32];
    arr.copy_from_slice(slice);
    Some(arr)
}
