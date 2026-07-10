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
    container_paths: Vec<String>,
    custom_strings: Vec<String>,
}

impl LogRedactor {
    pub fn new(container_paths: Vec<String>, custom_strings: Vec<String>) -> Self {
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
            container_paths,
            custom_strings,
        }
    }

    fn redact_custom_strings<'a>(&self, input: &'a str) -> Cow<'a, str> {
        // Can probably me made a lot faster with aho-corasick if optimization is ever needed.
        let mut out = Cow::from(input);
        for redact in &self.custom_strings {
            out = out.replace(redact, "[REDACTED]").into()
        }
        out
    }

    pub fn add_custom_string(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return;
        }
        if !self.custom_strings.iter().any(|s| s == trimmed) {
            self.custom_strings.push(trimmed.to_string());
        }
    }

    /// Redact sensitive information from the input string.
    ///
    /// Returns `Cow::Borrowed(input)` when nothing was redacted (zero allocation).
    /// Returns `Cow::Owned(...)` when at least one redaction was applied.
    pub fn redact(&self, input: &str) -> Option<String> {
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
        if let Cow::Owned(s) = self.redact_custom_strings(current!(owned, input)) {
            owned = Some(s);
        }

        owned
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
    // SAFETY:
    // `paths` is either null or a pointer to `paths_count` null-terminated C-strings.
    let container_paths = unsafe { ptr_array_to_vec(paths, paths_count) };
    Box::into_raw(Box::new(LogRedactor::new(container_paths, Vec::new())))
}

/// Converts a C array of C string pointers into a `Vec<String>`.
///
/// Strings must be valid UTF-8, null entries and invalid UTF-8 strings are ignored.
/// Returns an empty vector if `ptr` is null or `count == 0`.
///
/// # Safety
/// - `ptr` must be either null or point to a valid array of `count` pointers.
/// - Each element must be either null or a valid, null-terminated C string.
/// - All pointers must be valid for reads for the duration of this call.
unsafe fn ptr_array_to_vec(ptr: *const *const libc::c_char, count: usize) -> Vec<String> {
    if ptr.is_null() || count == 0 {
        return Vec::new();
    }

    // SAFETY: as per the caller docs, `ptr` must be a valid array for at least `count` pointers
    unsafe { std::slice::from_raw_parts(ptr, count) }
        .iter()
        .filter_map(|&p| {
            if p.is_null() {
                return None;
            }

            // SAFETY:
            // - `p` must point to a valid, null-terminated C string.
            // - The pointed-to string must contain valid UTF-8.
            unsafe { CStr::from_ptr(p) }
                .to_str()
                .ok()
                .map(str::to_owned)
        })
        .collect()
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

    // SAFETY: `redactor` pointer must point to a valid `LogRedactor` instance
    let redactor = unsafe { &*redactor };
    // SAFETY: `input` must be a valid C string pointer
    let input_str = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let result = redactor.redact(input_str);
    match result {
        None => std::ptr::null_mut(),
        Some(s) => match CString::new(s) {
            Ok(cstring) => cstring.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
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
        // SAFETY: `ptr` must be a valid C string pointer
        drop(unsafe { CString::from_raw(ptr) });
    }
}

/// Add a custom string to the log redactor.
///
/// # Safety
/// - `redactor` must be a valid pointer returned by `create_log_redactor`.
/// - `input` must be a valid pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub extern "C" fn log_redactor_add_custom_string(
    redactor: *mut LogRedactor,
    input: *const libc::c_char,
) {
    if redactor.is_null() || input.is_null() {
        return;
    }

    // SAFETY: `redactor` must be a valid pointer to a [`LogRedactor`] instance
    let redactor = unsafe { &mut *redactor };
    // SAFETY: `input` must be a valid C string pointer
    let Ok(input_str) = unsafe { CStr::from_ptr(input) }.to_str() else {
        return;
    };

    redactor.add_custom_string(input_str);
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
        // SAFETY: `redactor` must be a valid pointer to a `Box<LogRedactor>`
        drop(unsafe { Box::from_raw(redactor) });
    }
}
