//! FFI log redaction that mirrors Swift's LogRedactor.
//!
//! This module provides a Rust implementation of log redaction that can be called
//! from Swift via FFI, allowing performance comparison between Rust and Swift regex.

use regex::Regex;
use std::ffi::{CStr, CString};
use std::sync::LazyLock;

const REDACTED: &str = "[REDACTED]";
const REDACTED_ACCOUNT: &str = "[REDACTED ACCOUNT NUMBER]";

/// IPv4 pattern from https://www.regular-expressions.info/ip.html
static IPV4_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b"
    ).unwrap()
});

/// IPv6 pattern from https://stackoverflow.com/a/17871737
static IPV6_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        (
        ([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|
        ([0-9a-fA-F]{1,4}:){1,7}:|
        ([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|
        ([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|
        ([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|
        ([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|
        ([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|
        [0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|
        :((:[0-9a-fA-F]{1,4}){1,7}|:)|
        fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|
        ::(ffff(:0{1,4}){0,1}:){0,1}
        ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
        (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|
        ([0-9a-fA-F]{1,4}:){1,4}:
        ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
        (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])
        )"
    ).unwrap()
});

/// Account number pattern: 16 consecutive digits
static ACCOUNT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{16}").unwrap());

/// Redact sensitive information from a C string.
///
/// Redacts:
/// - IPv4 addresses → `[REDACTED]`
/// - IPv6 addresses → `[REDACTED]`
/// - Account numbers (16-digit sequences) → `[REDACTED ACCOUNT NUMBER]`
///
/// # Safety
/// - `input` must be a valid pointer to a null-terminated UTF-8 string.
/// - The returned pointer must be freed by calling `redact_log_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redact_log(input: *const libc::c_char) -> *mut libc::c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let input_str = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Apply redactions in order: IPv4, IPv6, account numbers
    let result = IPV4_REGEX.replace_all(input_str, REDACTED);
    let result = IPV6_REGEX.replace_all(&result, REDACTED);
    let result = ACCOUNT_REGEX.replace_all(&result, REDACTED_ACCOUNT);

    match CString::new(result.into_owned()) {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by `redact_log`.
///
/// # Safety
/// - `ptr` must be a pointer returned by `redact_log`, or null.
/// - `ptr` must not have been freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redact_log_free(ptr: *mut libc::c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}
