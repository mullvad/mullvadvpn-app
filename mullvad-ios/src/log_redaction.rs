use crate::get_string;
use regex::Regex;
use std::borrow::Cow;
use std::ffi::c_char;
use std::ffi::CString;

#[repr(C)]
pub struct RedactionConfig {
    pub redact_account_numbers: bool,
    pub redact_ipv4: bool,
    pub redact_ipv6: bool,

    pub container_paths: *const *const c_char,
    pub container_paths_len: usize,

    pub custom_strings: *const *const c_char,
    pub custom_strings_len: usize,
}

#[repr(C)]
pub struct LogRedactor {
    ptr: *mut LogRedactorInner,
}
impl LogRedactor {
    unsafe fn inner(&self) -> &LogRedactorInner {
        unsafe { &*self.ptr }
    }
}

struct LogRedactorInner {
    account_number_regex: regex::Regex,
    ipv4_regex: regex::Regex,
    ipv6_regex: regex::Regex
}

impl LogRedactorInner {
    fn new() -> Self {
        Self {
            account_number_regex: Regex::new("\\d{16}").unwrap(),
            ipv4_regex: Regex::new(&Self::build_ipv4_regex()).unwrap(),
            ipv6_regex: Regex::new(&Self::build_ipv6_regex()).unwrap()
        }
    }

    fn redact_log(&self, input: &str, config: &RedactionConfig) -> Option<String> {
        let mut current = input.to_string();

        if config.redact_account_numbers { 
                current = self.redact_account_number(&current).into_owned();
        }

        if config.redact_ipv4 {
            current = self.redact_ipv4(&current).into_owned();
        }

        if config.redact_ipv6 {
            current = self.redact_ipv6(&current).into_owned();
        }

        let container_paths =
            unsafe { LogRedactorInner::to_vec(config.container_paths, config.container_paths_len) };

        let custom_strings =
            unsafe { LogRedactorInner::to_vec(config.custom_strings, config.custom_strings_len) };

        for path in &container_paths {
            current = current.replace(path, "[REDACTED CONTAINER PATH]");
        }

        for s in &custom_strings {
            current = current.replace(s, "[REDACTED]");
        }

        if current == input {
            None
        } else {
            Some(current)
        }
    }

    fn redact_account_number<'a>(&self, input: &'a str) -> Cow<'a, str> {
        self.account_number_regex
            .replace_all(input, "[REDACTED ACCOUNT NUMBER]")
    }

    fn redact_ipv4<'a>(&self, input: &'a str) -> Cow<'a, str> {
        self.ipv4_regex.replace_all(input, "[REDACTED]")
    }

    fn redact_ipv6<'a>(&self, input: &'a str) -> Cow<'a, str> {
        self.ipv6_regex.replace_all(input, "[REDACTED]")
    }

    fn build_ipv4_regex() -> String {
        // regex adapted from  https://www.regular-expressions.info/ip.html

        let above_250 = "25[0-5]";
        let above_200 = "2[0-4][0-9]";
        let above_100 = "1[0-9][0-9]";

        // 100-119 | 120-126 | 128-129 | 130 - 199
        let above_100_not_127 = "1(?:[01][0-9]|2[0-6]|2[89]|[3-9][0-9])";

        let above_0 = "0?[0-9][0-9]?";

        // matches 0-255, except 127
        let first_octet = format!("(?:{above_250}|{above_200}|{above_100_not_127}|{above_0})");

        // matches 0-255
        let ip_octet = format!("(?:{above_250}|{above_200}|{above_100}|{above_0})");

        format!("(?:{first_octet}\\.{ip_octet}\\.{ip_octet}\\.{ip_octet})")
    }

    fn build_ipv6_regex() -> String {
        // Regular expression obtained from:
        // https://stackoverflow.com/a/17871737
        let ipv4_segment = "(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])";
        let ipv4_address = format!("({ipv4_segment}\\.){{3,3}}{ipv4_segment}");

        let ipv6_segment = "[0-9a-fA-F]{1,4}";

        let long = format!("({ipv6_segment}:){{7,7}}{ipv6_segment}");
        let compressed_1 = format!("({ipv6_segment}:){{1,7}}:");
        let compressed_2 = format!("({ipv6_segment}:){{1,6}}:{ipv6_segment}");
        let compressed_3 = format!("({ipv6_segment}:){{1,5}}(:{ipv6_segment}){{1,2}}");
        let compressed_4 = format!("({ipv6_segment}:){{1,4}}(:{ipv6_segment}){{1,3}}");
        let compressed_5 = format!("({ipv6_segment}:){{1,3}}(:{ipv6_segment}){{1,4}}");
        let compressed_6 = format!("({ipv6_segment}:){{1,2}}(:{ipv6_segment}){{1,5}}");
        let compressed_7 = format!("{ipv6_segment}:((:{ipv6_segment}){{1,6}})");
        let compressed_8 = format!(":((:{ipv6_segment}){{1,7}}|:)");
        let link_local = "[Ff][Ee]80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}";
        let ipv4_mapped = format!("::([fF]{{4}}(:0{{1,4}}){{0,1}}:){{0,1}}{ipv4_address}");
        let ipv4_embedded = format!("({ipv6_segment}:){{1,4}}:{ipv4_address}");

        format!(
            "{long}|{link_local}|{ipv4_mapped}|{ipv4_embedded}|{compressed_8}|{compressed_7}|{compressed_6}|{compressed_5}|{compressed_4}|{compressed_3}|{compressed_2}|{compressed_1}",
        )
    }

    unsafe fn to_vec(ptr: *const *const c_char, len: usize) -> Vec<String> {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        slice
            .iter()
            .map(|&p| {
                unsafe { std::ffi::CStr::from_ptr(p) }
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init_log_redactor() -> LogRedactor {
    LogRedactor {
        ptr: Box::into_raw(Box::new(LogRedactorInner::new())),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn drop_log_redactor(redactor: LogRedactor) {
       if !redactor.ptr.is_null() {
        // SAFETY: `redactor.ptr` must be properly aligned and non-null
        // The caller must guarantee that `redactor.ptr` is not null and has not been freed
        unsafe {
            drop(Box::from_raw(redactor.ptr));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn redact_log(
    redactor: LogRedactor,
    input: *const c_char,
    config: *const RedactionConfig,
) -> *mut c_char {
    let log_str = unsafe { get_string(input) };
    let log_redactor = unsafe { redactor.inner() };
    let redaction_config = unsafe { &*config };
    match log_redactor.redact_log(&log_str, &redaction_config) {
        None => std::ptr::null_mut(),
        Some(cow_str) => std::ffi::CString::new(cow_str)
            .unwrap_or_default()
            .into_raw(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_rust_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}
