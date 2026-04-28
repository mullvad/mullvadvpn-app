use hyper::header::Entry;
use regex::Regex;
use std::borrow::Cow;
use std::ffi::c_char;

use crate::get_string;

#[repr(C)]
struct LogRedactor {
    ptr: *mut LogRedactorInner,
}
impl LogRedactor {
    unsafe fn inner(&self) -> &LogRedactorInner {
        unsafe { &*self.ptr }
    }
}

struct LogRedactorInner {
    account_number_regex: regex::Regex,
    network_info_regex: regex::Regex,
}

impl LogRedactorInner {
    fn new() -> Self {
        let boundary = "[^0-9a-zA-Z.:]";
        let combined_pattern = format!(
            "(?P<start>^|{})(?:{}|{}|{})",
            boundary,
            Self::build_ipv4_regex(),
            Self::build_ipv6_regex(),
            Self::build_mac_regex(),
        );

        Self {
            account_number_regex: Regex::new("\\d{16}").unwrap(),
            network_info_regex: Regex::new(&combined_pattern).unwrap(),
        }
    }

    fn redact_log(&self, log: &str) -> Option<String> {
        let redacted_log_entry = self.redact_account_number(log);
        let redacted_network_info = self.redact_network_info(&redacted_log_entry);

        let is_redacted = [&redacted_log_entry, &redacted_network_info]
            .iter()
            .any(|entry| matches!(entry, Cow::Owned(_)));
        if is_redacted {
            Some(redacted_network_info.to_string())
        } else {
            None
        }
    }

    fn redact_account_number<'a>(&self, input: &'a str) -> Cow<'a, str> {
        self.account_number_regex
            .replace_all(input, "[REDACTED ACCOUNT NUMBER]")
    }

    fn redact_network_info<'a>(&self, input: &'a str) -> Cow<'a, str> {
        self.network_info_regex
            .replace_all(input, "$start[REDACTED]")
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
    fn build_mac_regex() -> String {
        let octet = "[[:xdigit:]]{2}"; // 0 - ff

        // five pairs of two hexadecimal chars followed by colon or dash
        // followed by a pair of hexadecimal chars
        format!("(?:{octet}[:-]){{5}}({octet})")
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
    unsafe {
        Box::from_raw(redactor.ptr);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn redact_log(redactor: LogRedactor, log: *const c_char) -> *mut c_char {
    let log_str = unsafe { get_string(log) };
    let log_redactor = unsafe { redactor.inner() };
    match log_redactor.redact_log(&log_str) {
        None => std::ptr::null_mut(),
        Some(cow_str) => std::ffi::CString::new(cow_str).unwrap_or_default().into_raw(),
    }
}
