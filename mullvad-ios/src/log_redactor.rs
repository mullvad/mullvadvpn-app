//! Stateful log redaction exposed to Swift via FFI.
//!
//! `LogRedactor` holds compiled regexes and container paths, all immutable after construction.
//! Since there is no interior mutability, the struct is `Send + Sync` and safe to use
//! concurrently from multiple threads without locks.

use regex::Regex;
use std::borrow::Cow;
use std::ffi::{CStr, CString};

const REDACTED: &str = "[REDACTED]";
const REDACTED_ACCOUNT: &str = "[REDACTED ACCOUNT NUMBER]";
const REDACTED_CONTAINER_PATH: &str = "[REDACTED CONTAINER PATH]";

pub struct LogRedactor {
    ipv4_regex: Regex,
    ipv6_regex: Regex,
    account_regex: Regex,
    mac_regex: Regex,
    container_paths: Box<[String]>,
}

impl LogRedactor {
    pub fn new(container_paths: Vec<String>) -> Self {
        Self {
            ipv4_regex: Regex::new(
                r"\b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b",
            )
            .unwrap(),
            ipv6_regex: Regex::new(
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
                )",
            )
            .unwrap(),
            account_regex: Regex::new(r"\d{16}").unwrap(),
            mac_regex: Regex::new(r"(?i)(?:[0-9a-f]{2}[:-]){5}[0-9a-f]{2}").unwrap(),
            container_paths: container_paths.into_boxed_slice(),
        }
    }

    /// Redact sensitive information from the input string.
    ///
    /// Returns `Cow::Borrowed(input)` when nothing was redacted (zero allocation).
    /// Returns `Cow::Owned(...)` when at least one redaction was applied.
    pub fn redact<'a>(&self, input: &'a str) -> Cow<'a, str> {
        macro_rules! current {
            ($owned:expr, $input:expr) => {
                $owned.as_deref().unwrap_or($input)
            };
        }

        let mut owned: Option<String> = None;

        // Container path replacement first (simple string ops, cheapest)
        for path in self.container_paths.iter() {
            let s = current!(owned, input);
            if s.contains(path.as_str()) {
                owned = Some(s.replace(path.as_str(), REDACTED_CONTAINER_PATH));
            }
        }

        // Regex-based redaction — each step only allocates if there's a match
        if let Cow::Owned(s) = self
            .ipv4_regex
            .replace_all(current!(owned, input), REDACTED)
        {
            owned = Some(s);
        }
        if let Cow::Owned(s) = self
            .ipv6_regex
            .replace_all(current!(owned, input), REDACTED)
        {
            owned = Some(s);
        }
        if let Cow::Owned(s) = self
            .account_regex
            .replace_all(current!(owned, input), REDACTED_ACCOUNT)
        {
            owned = Some(s);
        }
        if let Cow::Owned(s) = self.mac_regex.replace_all(current!(owned, input), REDACTED) {
            owned = Some(s);
        }

        match owned {
            Some(s) => Cow::Owned(s),
            None => Cow::Borrowed(input),
        }
    }
}

/// Create a new log redactor with the given container paths.
///
/// # Safety
/// - `paths` must be a valid pointer to an array of `paths_count` pointers to null-terminated
///   UTF-8 strings, or null if `paths_count` is 0.
/// - The returned pointer must be freed by calling `log_redactor_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn create_log_redactor(
    paths: *const *const libc::c_char,
    paths_count: usize,
) -> *mut LogRedactor {
    let container_paths = if paths.is_null() || paths_count == 0 {
        Vec::new()
    } else {
        let path_ptrs = unsafe { std::slice::from_raw_parts(paths, paths_count) };
        path_ptrs
            .iter()
            .filter_map(|&ptr| {
                if ptr.is_null() {
                    return None;
                }
                unsafe { CStr::from_ptr(ptr) }
                    .to_str()
                    .ok()
                    .map(|s| s.to_owned())
            })
            .collect()
    };

    Box::into_raw(Box::new(LogRedactor::new(container_paths)))
}

/// Redact sensitive information from a string using the given redactor.
///
/// # Safety
/// - `redactor` must be a valid pointer returned by `create_log_redactor`.
/// - `input` must be a valid pointer to a null-terminated UTF-8 string.
/// - The returned pointer must be freed by calling `log_redactor_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn log_redactor_redact(
    redactor: *const LogRedactor,
    input: *const libc::c_char,
) -> *mut libc::c_char {
    if redactor.is_null() || input.is_null() {
        return std::ptr::null_mut();
    }

    let redactor = unsafe { &*redactor };
    let input_str = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let result = redactor.redact(input_str);
    match CString::new(result.into_owned()) {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by `log_redactor_redact`.
///
/// # Safety
/// - `ptr` must be a pointer returned by `log_redactor_redact`, or null.
/// - `ptr` must not have been freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn log_redactor_free_string(ptr: *mut libc::c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}

/// Free a log redactor created by `create_log_redactor`.
///
/// # Safety
/// - `redactor` must be a pointer returned by `create_log_redactor`, or null.
/// - `redactor` must not have been freed before.
/// - `redactor` must not be used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn log_redactor_free(redactor: *mut LogRedactor) {
    if !redactor.is_null() {
        drop(unsafe { Box::from_raw(redactor) });
    }
}
