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
    // SAFETY: `ptr` is non-null (checked above). Every caller is an FFI entry point
    // whose `# Safety` contract requires `ptr` to be either null or a valid
    // null-terminated C string, so it points to a readable `CStr` here.
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
    // SAFETY: `ptr` is non-null (checked above). Every caller is an FFI entry point
    // whose `# Safety` contract requires key pointers to reference at least 32
    // readable bytes, so the 32-byte slice is in bounds and outlives this read.
    let slice = unsafe { std::slice::from_raw_parts(ptr, 32) };
    let mut arr = [0u8; 32];
    arr.copy_from_slice(slice);
    Some(arr)
}
