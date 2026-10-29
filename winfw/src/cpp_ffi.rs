//! C FFI to the C++ `winfw.dll` implementation.
//!
//! Used by the snapshot tests as the reference implementation while the Rust
//! implementation in this crate is being written. The DLL is loaded at runtime
//! so that its location can be chosen with the `WINFW_DLL_PATH` environment
//! variable, and so that no import library is needed.
//!
//! The types and signatures mirror `windows/winfw/src/winfw/winfw.h` (and the
//! bindings in `talpid-core`) exactly.

use std::{
    ffi::{CStr, c_char, c_void},
    os::windows::ffi::OsStrExt,
    path::PathBuf,
    ptr,
};

use widestring::WideCString;
use windows_sys::Win32::Foundation::{FreeLibrary, HMODULE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

/// Errors from loading and calling into the C++ `winfw.dll`.
#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("Failed to load {path}")]
    LoadLibrary { path: PathBuf },
    #[error("winfw.dll has no export {symbol}")]
    MissingSymbol { symbol: &'static str },
}

/// The log sink installed while the module is initialized, see `MullvadLogSink` in
/// `windows/libshared/src/libshared/logging/logsink.h`.
type MullvadLogSink =
    unsafe extern "system" fn(level: MullvadLogLevel, message: *const c_char, context: *mut c_void);

/// Log levels of libshared, see `MULLVAD_LOG_LEVEL` in
/// `windows/libshared/src/libshared/logging/logsink.h`.
#[repr(u32)]
pub enum MullvadLogLevel {
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

/// The log sink context, passed as an opaque pointer. WinFW only forwards it.
const LOG_CONTEXT: &[u8] = b"WinFwSnapshots\0";

/// Whether DHCP requests are permitted, and private networks are reachable.
/// `WinFwSettings` in `winfw.h`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WinFwSettings {
    pub permit_dhcp: bool,
    pub permit_lan: bool,
}

impl WinFwSettings {
    pub fn new(permit_lan: bool) -> Self {
        Self {
            permit_dhcp: true,
            permit_lan,
        }
    }
}

/// `WinFwProtocol` in `winfw.h`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinFwProtocol {
    Tcp = 0,
    Udp = 1,
}

/// `WinFwEndpoint` in `winfw.h`.
#[repr(C)]
pub struct WinFwEndpoint {
    pub ip: *const u16,
    pub port: u16,
    pub protocol: WinFwProtocol,
}

/// `WinFwAllowedEndpoint` in `winfw.h`.
#[repr(C)]
pub struct WinFwAllowedEndpoint {
    pub num_clients: u32,
    pub clients: *const *const u16,
    pub endpoint: WinFwEndpoint,
}

/// `WinFwAllowedTunnelTrafficType` in `winfw.h`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinFwAllowedTunnelTrafficType {
    None = 0,
    All = 1,
    One = 2,
    Two = 3,
}

/// `WinFwAllowedTunnelTraffic` in `winfw.h`.
#[repr(C)]
pub struct WinFwAllowedTunnelTraffic {
    pub traffic_type: WinFwAllowedTunnelTrafficType,
    pub endpoint1: *const WinFwEndpoint,
    pub endpoint2: *const WinFwEndpoint,
}

/// `WINFW_CLEANUP_POLICY` in `winfw.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinFwCleanupPolicy {
    ResetFirewall = 1,
    BlockingUntilReboot = 2,
}

/// `WINFW_ACTIVE_POLICY` in `winfw.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinFwActivePolicy {
    None = 0,
    Connecting = 1,
    Connected = 2,
    Blocked = 3,
}

impl WinFwActivePolicy {
    fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Blocked,
            _ => Self::None,
        }
    }
}

/// `WINFW_POLICY_STATUS` in `winfw.h`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinFwPolicyStatus {
    Success = 0,
    GeneralFailure = 1,
    LockTimeout = 2,
}

impl WinFwPolicyStatus {
    fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Success,
            2 => Self::LockTimeout,
            _ => Self::GeneralFailure,
        }
    }
}

/// An endpoint (IP, port, protocol) plus its allowed client processes, with owned
/// wide strings backing the pointers passed over the FFI boundary.
///
/// A `WinFwEndpoint` view of this, or references to the endpoint, are only valid
/// while this argument is alive: pass them as arguments, never store them.
pub struct EndpointArgument {
    ip: WideCString,
    /// The IP as passed in, for the Rust implementation driver.
    pub(crate) ip_str: String,
    clients: Vec<WideCString>,
    /// The clients as passed in, for the Rust implementation driver.
    pub(crate) clients_str: Vec<String>,
    client_ptrs: Vec<*const u16>,
    port: u16,
    protocol: WinFwProtocol,
}

impl EndpointArgument {
    pub fn new(ip: &str, port: u16, protocol: WinFwProtocol, clients: &[&str]) -> Self {
        let clients: Vec<WideCString> = clients
            .iter()
            .map(|client| WideCString::from_str_truncate(*client))
            .collect();
        let client_ptrs = clients.iter().map(|w| w.as_ptr()).collect();
        let clients_str = clients
            .iter()
            .map(|client| client.to_string_lossy())
            .collect();
        Self {
            ip: WideCString::from_str_truncate(ip),
            ip_str: ip.to_owned(),
            clients,
            clients_str,
            client_ptrs,
            port,
            protocol,
        }
    }

    /// The IP address, as passed in.
    pub fn ip(&self) -> &str {
        &self.ip_str
    }

    /// The port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The transport protocol.
    pub fn protocol(&self) -> WinFwProtocol {
        self.protocol
    }

    /// The allowed client processes, as passed in.
    pub fn clients(&self) -> &[String] {
        &self.clients_str
    }

    /// A `WinFwEndpoint` view valid while `self` is alive.
    fn endpoint(&self) -> WinFwEndpoint {
        WinFwEndpoint {
            ip: self.ip.as_ptr(),
            port: self.port,
            protocol: self.protocol,
        }
    }

    /// A `WinFwAllowedEndpoint` view valid while `self` is alive.
    fn allowed_endpoint(&self) -> WinFwAllowedEndpoint {
        WinFwAllowedEndpoint {
            num_clients: self.clients.len() as u32,
            clients: self.client_ptrs.as_ptr(),
            endpoint: self.endpoint(),
        }
    }
}

/// Allowed tunnel traffic argument, see `WinFwAllowedTunnelTraffic` in `winfw.h`.
pub struct AllowedTunnelTraffic<'a> {
    pub traffic_type: WinFwAllowedTunnelTrafficType,
    pub endpoint1: Option<&'a EndpointArgument>,
    pub endpoint2: Option<&'a EndpointArgument>,
}

/// Function pointer types for the DLL exports, `__stdcall` (`WINFW_API`).
type InitializeFn = unsafe extern "system" fn(u32, Option<MullvadLogSink>, *mut c_void) -> bool;
type InitializeBlockedFn = unsafe extern "system" fn(
    u32,
    *const WinFwSettings,
    *const WinFwAllowedEndpoint,
    Option<MullvadLogSink>,
    *mut c_void,
) -> bool;
type DeinitializeFn = unsafe extern "system" fn(WinFwCleanupPolicy) -> bool;
type ActivePolicyFn = unsafe extern "system" fn() -> u32;
type ApplyConnectingFn = unsafe extern "system" fn(
    *const WinFwSettings,
    usize,
    *const WinFwEndpoint,
    *const u16,
    *const *const u16,
    usize,
    *const u16,
    *const WinFwAllowedEndpoint,
    *const WinFwAllowedTunnelTraffic,
) -> u32;
type ApplyConnectedFn = unsafe extern "system" fn(
    *const WinFwSettings,
    usize,
    *const WinFwEndpoint,
    *const u16,
    *const *const u16,
    usize,
    *const u16,
    *const *const u16,
    usize,
    *const *const u16,
    usize,
) -> u32;
type ApplyBlockedFn =
    unsafe extern "system" fn(*const WinFwSettings, *const WinFwAllowedEndpoint) -> u32;
type ResetFn = unsafe extern "system" fn() -> u32;

/// A loaded `winfw.dll`, ready to be called.
pub struct CppWinFw {
    module: HMODULE,
}

impl CppWinFw {
    /// Load `winfw.dll` from `WINFW_DLL_PATH`, or from the usual build output
    /// locations around the running executable.
    pub fn load_default() -> Result<Self, LoadError> {
        if let Some(path) = std::env::var_os("WINFW_DLL_PATH") {
            return Self::load(&PathBuf::from(path));
        }

        if let Some(path) = default_dll_path() {
            return Self::load(&path);
        }

        // Last resort: let Windows search for a bare "winfw.dll" (application
        // directory and PATH).
        Self::load(&PathBuf::from("winfw.dll"))
    }

    pub fn load(path: &std::path::Path) -> Result<Self, LoadError> {
        let mut wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: `wide` is a valid, null-terminated path for the duration of the call.
        let module = unsafe { LoadLibraryW(wide.as_mut_ptr()) };
        if module.is_null() || module == INVALID_HANDLE_VALUE {
            return Err(LoadError::LoadLibrary {
                path: path.to_path_buf(),
            });
        }
        Ok(Self { module })
    }

    /// Look up an exported function and transmute it to its typed signature.
    fn function<FnPtr>(&self, name: &'static str) -> Result<FnPtr, LoadError>
    where
        FnPtr: Copy,
    {
        // `GetProcAddress` takes a null-terminated ANSI string.
        let name_bytes: Vec<u8> = name.bytes().chain(std::iter::once(0)).collect();
        // SAFETY: The module handle is valid, the name is null-terminated, and a
        // null return is checked.
        let sym = unsafe { GetProcAddress(self.module, name_bytes.as_ptr()) };
        if sym.is_none() {
            return Err(LoadError::MissingSymbol { symbol: name });
        }
        // SAFETY: `FnPtr` is only ever a function pointer type matching the DLL's
        // exported `__stdcall` signature for `name`.
        Ok(unsafe { std::mem::transmute_copy(&sym) })
    }

    /// `WinFw_Initialize`.
    pub fn initialize(&mut self, timeout_seconds: u32) -> Result<(), LoadError> {
        let fun: InitializeFn = self.function("WinFw_Initialize")?;
        // SAFETY: The signature matches the export; the context pointer stays
        // valid for the rest of the process.
        let result = unsafe {
            fun(
                timeout_seconds,
                Some(log_sink),
                LOG_CONTEXT.as_ptr().cast_mut().cast::<c_void>(),
            )
        };
        assert!(result, "WinFw_Initialize failed");
        Ok(())
    }

    /// `WinFw_InitializeBlocked`.
    pub fn initialize_blocked(
        &mut self,
        timeout_seconds: u32,
        settings: &WinFwSettings,
        allowed_endpoint: Option<&EndpointArgument>,
    ) -> Result<(), LoadError> {
        let fun: InitializeBlockedFn = self.function("WinFw_InitializeBlocked")?;
        let ep = allowed_endpoint.map(|e| e.allowed_endpoint());
        let ep_ptr = ep.as_ref().map_or(ptr::null(), |e| e as *const _);
        // SAFETY: The signature matches the export; the endpoint (and every string
        // it points at) is alive for the duration of the call.
        let result = unsafe {
            fun(
                timeout_seconds,
                settings,
                ep_ptr,
                Some(log_sink),
                LOG_CONTEXT.as_ptr().cast_mut().cast::<c_void>(),
            )
        };
        assert!(result, "WinFw_InitializeBlocked failed");
        Ok(())
    }

    /// `WinFw_Deinitialize`.
    pub fn deinitialize(&mut self, cleanup: WinFwCleanupPolicy) -> Result<(), LoadError> {
        let fun: DeinitializeFn = self.function("WinFw_Deinitialize")?;
        // SAFETY: The signature matches the export.
        let result = unsafe { fun(cleanup) };
        assert!(result, "WinFw_Deinitialize failed");
        Ok(())
    }

    /// `WinFw_ActivePolicy`.
    pub fn active_policy(&self) -> Result<WinFwActivePolicy, LoadError> {
        let fun: ActivePolicyFn = self.function("WinFw_ActivePolicy")?;
        // SAFETY: The signature matches the export.
        Ok(WinFwActivePolicy::from_u32(unsafe { fun() }))
    }

    /// `WinFw_ApplyPolicyConnecting`.
    #[expect(clippy::too_many_arguments)]
    pub fn apply_policy_connecting(
        &mut self,
        settings: &WinFwSettings,
        relays: &[EndpointArgument],
        exit_endpoint_ip: Option<&str>,
        relay_clients: &[&str],
        tunnel_interface_alias: Option<&str>,
        allowed_endpoint: &EndpointArgument,
        allowed_tunnel_traffic: AllowedTunnelTraffic<'_>,
    ) -> Result<WinFwPolicyStatus, LoadError> {
        let fun: ApplyConnectingFn = self.function("WinFw_ApplyPolicyConnecting")?;

        let endpoints: Vec<WinFwEndpoint> = relays.iter().map(|e| e.endpoint()).collect();

        let exit_ip_w = exit_endpoint_ip.map(WideCString::from_str_truncate);
        let exit_ip_ptr = exit_ip_w.as_ref().map_or(ptr::null(), |w| w.as_ptr());

        let client_wstrs: Vec<WideCString> = relay_clients
            .iter()
            .map(|c| WideCString::from_str_truncate(*c))
            .collect();
        let client_ptrs: Vec<*const u16> = client_wstrs.iter().map(|w| w.as_ptr()).collect();

        let tunnel_w = tunnel_interface_alias.map(WideCString::from_str_truncate);
        let tunnel_ptr = tunnel_w.as_ref().map_or(ptr::null(), |w| w.as_ptr());

        let allowed = allowed_endpoint.allowed_endpoint();

        let att_endpoints = (
            allowed_tunnel_traffic.endpoint1.map(|e| e.endpoint()),
            allowed_tunnel_traffic.endpoint2.map(|e| e.endpoint()),
        );
        let att = WinFwAllowedTunnelTraffic {
            traffic_type: allowed_tunnel_traffic.traffic_type,
            endpoint1: att_endpoints
                .0
                .as_ref()
                .map_or(ptr::null(), |e| e as *const _),
            endpoint2: att_endpoints
                .1
                .as_ref()
                .map_or(ptr::null(), |e| e as *const _),
        };

        // SAFETY: The signature matches the export, and every pointer stays valid
        // for the duration of the call.
        let status = unsafe {
            fun(
                settings,
                endpoints.len(),
                endpoints.as_ptr(),
                exit_ip_ptr,
                client_ptrs.as_ptr(),
                client_ptrs.len(),
                tunnel_ptr,
                &raw const allowed,
                &raw const att,
            )
        };

        // Keep everything the DLL may still reference alive until here.
        drop(endpoints);
        drop(client_wstrs);
        drop(exit_ip_w);
        drop(tunnel_w);
        Ok(WinFwPolicyStatus::from_u32(status))
    }

    /// `WinFw_ApplyPolicyConnected`.
    #[expect(clippy::too_many_arguments)]
    pub fn apply_policy_connected(
        &mut self,
        settings: &WinFwSettings,
        relays: &[EndpointArgument],
        exit_endpoint_ip: Option<&str>,
        relay_clients: &[&str],
        tunnel_interface_alias: &str,
        tunnel_dns_servers: &[&str],
        non_tunnel_dns_servers: &[&str],
    ) -> Result<WinFwPolicyStatus, LoadError> {
        let fun: ApplyConnectedFn = self.function("WinFw_ApplyPolicyConnected")?;

        let endpoints: Vec<WinFwEndpoint> = relays.iter().map(|e| e.endpoint()).collect();

        let exit_ip_w = exit_endpoint_ip.map(WideCString::from_str_truncate);
        let exit_ip_ptr = exit_ip_w.as_ref().map_or(ptr::null(), |w| w.as_ptr());

        let client_wstrs: Vec<WideCString> = relay_clients
            .iter()
            .map(|c| WideCString::from_str_truncate(*c))
            .collect();
        let client_ptrs: Vec<*const u16> = client_wstrs.iter().map(|w| w.as_ptr()).collect();

        let tunnel_w = WideCString::from_str_truncate(tunnel_interface_alias);

        let tunnel_dns_w: Vec<WideCString> = tunnel_dns_servers
            .iter()
            .map(|ip| WideCString::from_str_truncate(*ip))
            .collect();
        let tunnel_dns_ptrs: Vec<*const u16> = tunnel_dns_w.iter().map(|w| w.as_ptr()).collect();

        let non_tunnel_dns_w: Vec<WideCString> = non_tunnel_dns_servers
            .iter()
            .map(|ip| WideCString::from_str_truncate(*ip))
            .collect();
        let non_tunnel_dns_ptrs: Vec<*const u16> =
            non_tunnel_dns_w.iter().map(|w| w.as_ptr()).collect();

        // SAFETY: The signature matches the export, and every pointer stays valid
        // for the duration of the call.
        let status = unsafe {
            fun(
                settings,
                endpoints.len(),
                endpoints.as_ptr(),
                exit_ip_ptr,
                client_ptrs.as_ptr(),
                client_ptrs.len(),
                tunnel_w.as_ptr(),
                tunnel_dns_ptrs.as_ptr(),
                tunnel_dns_ptrs.len(),
                non_tunnel_dns_ptrs.as_ptr(),
                non_tunnel_dns_ptrs.len(),
            )
        };

        // Keep everything the DLL may still reference alive until here.
        drop(endpoints);
        drop(client_wstrs);
        drop(exit_ip_w);
        drop(tunnel_dns_w);
        drop(non_tunnel_dns_w);
        drop(tunnel_w);
        Ok(WinFwPolicyStatus::from_u32(status))
    }

    /// `WinFw_ApplyPolicyBlocked`.
    pub fn apply_policy_blocked(
        &mut self,
        settings: &WinFwSettings,
        allowed_endpoint: Option<&EndpointArgument>,
    ) -> Result<WinFwPolicyStatus, LoadError> {
        let fun: ApplyBlockedFn = self.function("WinFw_ApplyPolicyBlocked")?;
        let ep = allowed_endpoint.map(|e| e.allowed_endpoint());
        let ep_ptr = ep.as_ref().map_or(ptr::null(), |e| e as *const _);
        // SAFETY: The signature matches the export, and the endpoint (and every
        // string it points at) is alive for the duration of the call.
        let status = unsafe { fun(settings, ep_ptr) };
        Ok(WinFwPolicyStatus::from_u32(status))
    }

    /// `WinFw_Reset`.
    pub fn reset(&mut self) -> Result<WinFwPolicyStatus, LoadError> {
        let fun: ResetFn = self.function("WinFw_Reset")?;
        // SAFETY: The signature matches the export.
        Ok(WinFwPolicyStatus::from_u32(unsafe { fun() }))
    }
}

impl Drop for CppWinFw {
    fn drop(&mut self) {
        // SAFETY: The handle came from `LoadLibraryW` and is only freed here.
        unsafe { FreeLibrary(self.module) };
    }
}

/// The log sink installed into the DLL: print to stderr.
unsafe extern "system" fn log_sink(
    level: MullvadLogLevel,
    message: *const c_char,
    _context: *mut c_void,
) {
    // SAFETY: The DLL passes a valid C string; a null cannot happen (see winfw.cpp).
    let message = if message.is_null() {
        "<null>".to_owned()
    } else {
        // SAFETY: the message pointer is checked above.
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned()
    };
    let level = match level {
        MullvadLogLevel::Error => "ERROR",
        MullvadLogLevel::Warning => "WARN",
        MullvadLogLevel::Info => "INFO",
        MullvadLogLevel::Debug => "DEBUG",
        MullvadLogLevel::Trace => "TRACE",
    };
    eprintln!("[winfw-cpp/{level}] {message}");
}

/// Find `winfw.dll` in the usual build output locations around the running
/// executable: next to the binary, and in the `target/{debug,release}` directories
/// above it (the test binary lives in `target/<mode>/deps/`).
fn default_dll_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    let target_dir = exe_dir
        .ancestors()
        .find(|dir| dir.ends_with("debug") || dir.ends_with("release"))?;
    [
        exe_dir.join("winfw.dll"),
        target_dir.join("winfw.dll"),
        target_dir.parent()?.join("winfw.dll"),
    ]
    .into_iter()
    .find(|path| path.exists())
}
