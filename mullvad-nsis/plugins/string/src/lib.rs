//! NSIS string plugin: string utilities for the Mullvad VPN installer.
//!
//! Exports:
//! - `Find` - find a substring position in a string

#![cfg(all(target_arch = "x86", target_os = "windows"))]

use nsis_plugin_api::{nsis_fn, popint, popstr, pushint};

// Find "searchString" "substring" begin_offset
//
// Returns the position of "substring" in "searchString" starting from begin_offset.
// If not found, returns -1. On error, pushes the NSIS error description.
#[nsis_fn]
fn Find() -> Result<(), Error> {
    // SAFETY: the `nsis_fn` wrapper called `exdll_init` to install the
    // NSIS stack globals before invoking this body, so the ABI helpers
    // (popstr/popint/pushint) are safe to call.
    let (search_string, substring, offset) = unsafe { (popstr()?, popstr()?, popint()?) };

    if offset < 0 || offset as usize > search_string.len() {
        // SAFETY: see above.
        return unsafe { pushint(-1) };
    }
    let offset = offset as usize;

    let result = match search_string[offset..].find(substring.as_str()) {
        Some(pos) => (offset + pos) as i32,
        None => -1,
    };
    // SAFETY: see above.
    unsafe { pushint(result) }
}
