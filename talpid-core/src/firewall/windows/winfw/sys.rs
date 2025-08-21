//! Data types and thin wrappers around WinFW C FFI.

use std::ffi::{CStr, c_char, c_void};
use std::io;
use std::ptr;

use windows_sys::Win32::Globalization::{CP_ACP, MultiByteToWideChar};

use super::{Error, WideCString};

pub const LOGGING_CONTEXT: &CStr = c"WinFw";

#[repr(C)]
#[allow(non_snake_case)]
pub struct WinFwSettings {
    permitDhcp: bool,
    permitLan: bool,
}

impl WinFwSettings {
    pub fn new(permit_lan: bool) -> WinFwSettings {
        WinFwSettings {
            permitDhcp: true,
            permitLan: permit_lan,
        }
    }
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum WinFwCleanupPolicy {
    ContinueBlocking = 0,
    ResetFirewall = 1,
    BlockingUntilReboot = 2,
}

#[derive(Debug)]
#[allow(dead_code)]
#[repr(u32)]
pub enum WinFwPolicyStatus {
    Success = 0,
    GeneralFailure = 1,
    LockTimeout = 2,
}

#[repr(C)]
pub struct WinFwEndpoint {
    pub ip: *const libc::wchar_t,
    pub port: u16,
    pub protocol: WinFwProt,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum WinFwProt {
    Tcp = 0u8,
    Udp = 1u8,
}

#[repr(C)]
pub struct WinFwAllowedEndpoint<'a> {
    num_clients: u32,
    clients: *const *const libc::wchar_t,
    endpoint: WinFwEndpoint,

    _phantom: std::marker::PhantomData<&'a WinFwAllowedEndpointContainer>,
}

pub struct WinFwAllowedEndpointContainer {
    pub _clients: Box<[WideCString]>,
    pub clients_ptrs: Box<[*const u16]>,
    pub ip: WideCString,
    pub port: u16,
    pub protocol: WinFwProt,
}

impl WinFwAllowedEndpointContainer {
    pub fn as_endpoint(&self) -> WinFwAllowedEndpoint<'_> {
        WinFwAllowedEndpoint {
            num_clients: self.clients_ptrs.len() as u32,
            clients: self.clients_ptrs.as_ptr(),
            endpoint: WinFwEndpoint {
                ip: self.ip.as_ptr(),
                port: self.port,
                protocol: self.protocol,
            },

            _phantom: std::marker::PhantomData,
        }
    }
}

#[repr(C)]
pub struct WinFwAllowedTunnelTraffic {
    pub type_: WinFwAllowedTunnelTrafficType,
    pub endpoint1: *const WinFwEndpoint,
    pub endpoint2: *const WinFwEndpoint,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum WinFwAllowedTunnelTrafficType {
    None,
    All,
    One,
    Two,
}

ffi_error!(InitializationResult, Error::Initialization);
ffi_error!(DeinitializationResult, Error::Deinitialization);

unsafe extern "system" {
    #[link_name = "WinFw_Initialize"]
    pub fn WinFw_Initialize(
        timeout: libc::c_uint,
        sink: Option<LogSink>,
        sink_context: *const c_char,
    ) -> InitializationResult;

    #[link_name = "WinFw_InitializeBlocked"]
    pub fn WinFw_InitializeBlocked(
        timeout: libc::c_uint,
        settings: &WinFwSettings,
        allowed_endpoint: *const WinFwAllowedEndpoint<'_>,
        sink: Option<LogSink>,
        sink_context: *const c_char,
    ) -> InitializationResult;

    #[link_name = "WinFw_Deinitialize"]
    pub fn WinFw_Deinitialize(cleanupPolicy: WinFwCleanupPolicy) -> DeinitializationResult;

    #[link_name = "WinFw_ApplyPolicyConnecting"]
    pub fn WinFw_ApplyPolicyConnecting(
        settings: &WinFwSettings,
        relay: &WinFwEndpoint,
        exitEndpointIp: *const libc::wchar_t,
        relayClient: *const *const libc::wchar_t,
        relayClientLen: usize,
        tunnelIfaceAlias: *const libc::wchar_t,
        allowedEndpoint: *const WinFwAllowedEndpoint<'_>,
        allowedTunnelTraffic: &WinFwAllowedTunnelTraffic,
    ) -> WinFwPolicyStatus;

    #[link_name = "WinFw_ApplyPolicyConnected"]
    pub fn WinFw_ApplyPolicyConnected(
        settings: &WinFwSettings,
        relay: &WinFwEndpoint,
        exitEndpointIp: *const libc::wchar_t,
        relayClient: *const *const libc::wchar_t,
        relayClientLen: usize,
        tunnelIfaceAlias: *const libc::wchar_t,
        tunnelDnsServers: *const *const libc::wchar_t,
        numTunnelDnsServers: usize,
        nonTunnelDnsServers: *const *const libc::wchar_t,
        numNonTunnelDnsServers: usize,
    ) -> WinFwPolicyStatus;

    #[link_name = "WinFw_ApplyPolicyBlocked"]
    pub fn WinFw_ApplyPolicyBlocked(
        settings: &WinFwSettings,
        allowed_endpoint: *const WinFwAllowedEndpoint<'_>,
    ) -> WinFwPolicyStatus;

    #[link_name = "WinFw_Reset"]
    pub fn WinFw_Reset() -> WinFwPolicyStatus;
}

pub type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);

/// Logging callback implementation.
///
/// SAFETY:
/// - `msg` must point to a valid C string or be null.
/// - `context` must point to a valid C string or be null.
pub extern "system" fn log_sink(
    level: log::Level,
    msg: *const std::ffi::c_char,
    context: *mut std::ffi::c_void,
) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
        return;
    }

    let target = if context.is_null() {
        "UNKNOWN".into()
    } else {
        // SAFETY: context is not null & caller promise that context is a valid C string.
        unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
    };

    // SAFETY: msg is not null & caller promise that msg is a valid C string.
    let mb_string = unsafe { CStr::from_ptr(msg) };

    let managed_msg = match multibyte_to_wide(mb_string, CP_ACP) {
        Ok(wide_str) => String::from_utf16_lossy(&wide_str),
        // Best effort:
        Err(_) => mb_string.to_string_lossy().into_owned(),
    };

    log::logger().log(
        &log::Record::builder()
            .level(level)
            .target(&target)
            .args(format_args!("{managed_msg}"))
            .build(),
    );
}

/// Convert `mb_string`, with the given character encoding `codepage`, to a UTF-16 string.
pub fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, io::Error> {
    if mb_string.is_empty() {
        return Ok(vec![]);
    }

    // SAFETY: `mb_string` is null-terminated and valid.
    let wc_size = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            ptr::null_mut(),
            0,
        )
    };

    if wc_size == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wc_buffer = vec![0u16; usize::try_from(wc_size).unwrap()];

    // SAFETY: `wc_buffer` can contain up to `wc_size` characters, including a null
    // terminator.
    let chars_written = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            wc_buffer.as_mut_ptr(),
            wc_size,
        )
    };

    if chars_written == 0 {
        return Err(io::Error::last_os_error());
    }

    wc_buffer.truncate(usize::try_from(chars_written - 1).unwrap());

    Ok(wc_buffer)
}

#[cfg(test)]
mod test {
    use super::*;
    use windows_sys::Win32::Globalization::CP_UTF8;

    #[test]
    fn test_multibyte_to_wide() {
        // € = 0x20AC in UTF-16
        let converted = multibyte_to_wide(c"€€", CP_UTF8);
        const EXPECTED: &[u16] = &[0x20AC, 0x20AC];
        assert!(
            matches!(converted.as_deref(), Ok(EXPECTED)),
            "expected Ok({EXPECTED:?}), got {converted:?}",
        );

        // boundary case
        let converted = multibyte_to_wide(c"", CP_UTF8);
        assert!(
            matches!(converted.as_deref(), Ok([])),
            "unexpected result {converted:?}"
        );
    }
}
