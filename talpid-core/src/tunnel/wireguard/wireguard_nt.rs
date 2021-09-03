use super::{
    config::Config,
    logging,
    stats::{Stats, StatsMap},
    Tunnel,
};
use bitflags::bitflags;
use ipnetwork::IpNetwork;
use lazy_static::lazy_static;
use std::{
    ffi::CStr,
    fmt, io, iter, mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::windows::{ffi::OsStrExt, io::RawHandle},
    path::Path,
    ptr,
    sync::{Arc, Mutex},
};
use talpid_types::ErrorExt;
use widestring::{U16CStr, U16CString};
use winapi::{
    shared::{
        guiddef::GUID,
        ifdef::NET_LUID,
        in6addr::IN6_ADDR,
        inaddr::IN_ADDR,
        minwindef::{BOOL, FARPROC, HINSTANCE, HMODULE},
        winerror::ERROR_MORE_DATA,
        ws2def::{ADDRESS_FAMILY, AF_INET, AF_INET6},
        ws2ipdef::SOCKADDR_INET,
    },
    um::libloaderapi::{
        FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH,
    },
};


lazy_static! {
    static ref WG_NT_DLL: Mutex<Option<Arc<WgNtDll>>> = Mutex::new(None);
    static ref ADAPTER_POOL: U16CString = U16CString::from_str("Mullvad").unwrap();
    static ref ADAPTER_ALIAS: U16CString = U16CString::from_str("Mullvad").unwrap();
}

const ADAPTER_GUID: GUID = GUID {
    Data1: 0x514a3988,
    Data2: 0x9716,
    Data3: 0x43d5,
    Data4: [0x8b, 0x05, 0x31, 0xda, 0x25, 0xa0, 0x44, 0xa9],
};

/// Longest possible adapter name (in characters), including null terminator
const MAX_ADAPTER_NAME: usize = 128;

type WireGuardOpenAdapterFn =
    unsafe extern "stdcall" fn(pool: *const u16, name: *const u16) -> RawHandle;
type WireGuardCreateAdapterFn = unsafe extern "stdcall" fn(
    pool: *const u16,
    name: *const u16,
    requested_guid: *const GUID,
    reboot_required: *mut BOOL,
) -> RawHandle;
type WireGuardFreeAdapterFn = unsafe extern "stdcall" fn(adapter: RawHandle);
type WireGuardDeleteAdapterFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, reboot_required: *mut BOOL) -> BOOL;
type WireGuardGetAdapterLuidFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, luid: *mut NET_LUID);
type WireGuardGetAdapterNameFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, name: *mut u16) -> BOOL;
type WireGuardSetConfigurationFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, config: *const u8, bytes: u32) -> BOOL;
type WireGuardGetConfigurationFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, config: *const u8, bytes: *mut u32) -> BOOL;
type WireGuardSetStateFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, state: WgAdapterState) -> BOOL;

#[cfg(windows)]
#[repr(C)]
#[allow(dead_code)]
enum LogLevel {
    Info = 0,
    Warn = 1,
    Err = 2,
}

#[cfg(windows)]
impl From<LogLevel> for logging::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Info => Self::Info,
            LogLevel::Warn => Self::Warning,
            LogLevel::Err => Self::Error,
        }
    }
}

type WireGuardLoggerCb = extern "stdcall" fn(LogLevel, timestamp: u64, *const u16);
type WireGuardSetLoggerFn = extern "stdcall" fn(Option<WireGuardLoggerCb>);

#[repr(C)]
#[allow(dead_code)]
enum WireGuardAdapterLogState {
    Off = 0,
    On = 1,
    OnWithPrefix = 2,
}

type WireGuardSetAdapterLoggingFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, state: WireGuardAdapterLogState) -> BOOL;

type RebootRequired = bool;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to load WireGuardNT
    #[error(display = "Failed to load wireguard.dll")]
    DllError(#[error(source)] io::Error),

    /// Failed to remove tunnel interface
    #[error(display = "Failed to remove residual tunnel device")]
    DeleteExistingTunnelError(#[error(source)] io::Error),

    /// Failed to create tunnel interface
    #[error(display = "Failed to create WireGuard device")]
    CreateTunnelDeviceError(#[error(source)] io::Error),

    /// Failed to delete tunnel interface
    #[error(display = "Failed to delete WireGuard device")]
    DeleteTunnelDeviceError(#[error(source)] io::Error),

    /// Failed to obtain tunnel interface alias
    #[error(display = "Failed to obtain interface name")]
    ObtainAliasError(#[error(source)] io::Error),

    /// Failed to get WireGuard tunnel config for device
    #[error(display = "Failed to get tunnel WireGuard config")]
    GetWireGuardConfigError(#[error(source)] io::Error),

    /// Failed to set WireGuard tunnel config on device
    #[error(display = "Failed to set tunnel WireGuard config")]
    SetWireGuardConfigError(#[error(source)] io::Error),

    /// Failed to set MTU on tunnel device
    #[error(display = "Failed to set tunnel IPv4 interface MTU")]
    SetTunnelIpv4MtuError(#[error(source)] io::Error),

    /// Failed to set MTU on tunnel device
    #[error(display = "Failed to set tunnel IPv6 interface MTU")]
    SetTunnelIpv6MtuError(#[error(source)] io::Error),

    /// Failed to set the tunnel state to up
    #[error(display = "Failed to enable the tunnel adapter")]
    EnableTunnelError(#[error(source)] io::Error),

    /// Unknown address family
    #[error(display = "Unknown address family: {}", _0)]
    UnknownAddressFamily(i32),

    /// Failure to set up logging
    #[error(display = "Failed to set up logging")]
    InitLoggingError(#[error(source)] logging::Error),

    /// Invalid allowed IP
    #[error(display = "Invalid CIDR prefix")]
    InvalidAllowedIpCidr,

    /// Allowed IP contains non-zero host bits
    #[error(display = "Allowed IP contains non-zero host bits")]
    InvalidAllowedIpBits,

    /// Failed to parse data returned by the driver
    #[error(display = "Failed to parse data returned by wireguard-nt")]
    InvalidConfigData,
}

pub struct WgNtTunnel {
    device: Option<WgNtAdapter>,
    interface_luid: NET_LUID,
    interface_name: String,
    _logger_handle: LoggerHandle,
}

const WIREGUARD_KEY_LENGTH: usize = 32;

/// See `WIREGUARD_ALLOWED_IP` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Clone, Copy)]
#[repr(C, align(8))]
union WgIpAddr {
    v4: IN_ADDR,
    v6: IN6_ADDR,
}

/// See `WIREGUARD_ALLOWED_IP` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Clone, Copy)]
#[repr(C, align(8))]
struct WgAllowedIp {
    address: WgIpAddr,
    address_family: ADDRESS_FAMILY,
    cidr: u8,
}

impl WgAllowedIp {
    fn new(address: WgIpAddr, address_family: ADDRESS_FAMILY, cidr: u8) -> Result<Self> {
        Self::validate(&address, address_family, cidr)?;
        Ok(Self {
            address,
            address_family,
            cidr,
        })
    }

    fn validate(address: &WgIpAddr, address_family: ADDRESS_FAMILY, cidr: u8) -> Result<()> {
        match address_family as i32 {
            AF_INET => {
                if cidr > 32 {
                    return Err(Error::InvalidAllowedIpCidr);
                }
                let host_mask = u32::MAX.checked_shr(u32::from(cidr)).unwrap_or(0);
                if host_mask & (unsafe { *(address.v4.S_un.S_addr()) }.to_be()) != 0 {
                    return Err(Error::InvalidAllowedIpBits);
                }
            }
            AF_INET6 => {
                if cidr > 128 {
                    return Err(Error::InvalidAllowedIpCidr);
                }
                let mut host_mask = u128::MAX.checked_shr(u32::from(cidr)).unwrap_or(0);
                let bytes = unsafe { address.v6.u.Byte() };
                for byte in bytes.iter().rev() {
                    if byte & ((host_mask & 0xff) as u8) != 0 {
                        return Err(Error::InvalidAllowedIpBits);
                    }
                    host_mask = host_mask >> 8;
                }
            }
            family => return Err(Error::UnknownAddressFamily(family)),
        }
        Ok(())
    }
}

impl PartialEq for WgAllowedIp {
    fn eq(&self, other: &Self) -> bool {
        if self.cidr != other.cidr {
            return false;
        }
        match self.address_family as i32 {
            AF_INET => {
                inaddr_to_ipaddr(unsafe { self.address.v4 })
                    == inaddr_to_ipaddr(unsafe { other.address.v4 })
            }
            AF_INET6 => {
                in6addr_to_ipaddr(unsafe { self.address.v6 })
                    == in6addr_to_ipaddr(unsafe { other.address.v6 })
            }
            _ => {
                log::error!("Allowed IP uses unknown address family");
                true
            }
        }
    }
}
impl Eq for WgAllowedIp {}

impl fmt::Debug for WgAllowedIp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("WgAllowedIp");
        match self.address_family as i32 {
            AF_INET => s.field("address", &inaddr_to_ipaddr(unsafe { self.address.v4 })),
            AF_INET6 => s.field("address", &in6addr_to_ipaddr(unsafe { self.address.v6 })),
            _ => s.field("address", &"<unknown>"),
        };
        s.field("address_family", &self.address_family)
            .field("cidr", &self.cidr)
            .finish()
    }
}

bitflags! {
    /// See `WIREGUARD_PEER_FLAG` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
    struct WgPeerFlag: u32 {
        const HAS_PUBLIC_KEY = 0b00000001;
        const HAS_PRESHARED_KEY = 0b00000010;
        const HAS_PERSISTENT_KEEPALIVE = 0b00000100;
        const HAS_ENDPOINT = 0b00001000;
        const REPLACE_ALLOWED_IPS = 0b00100000;
        const REMOVE = 0b01000000;
        const UPDATE = 0b10000000;
    }
}

/// See `WIREGUARD_PEER` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C, align(8))]
struct WgPeer {
    flags: WgPeerFlag,
    reserved: u32,
    public_key: [u8; WIREGUARD_KEY_LENGTH],
    preshared_key: [u8; WIREGUARD_KEY_LENGTH],
    persistent_keepalive: u16,
    endpoint: SockAddrInet,
    tx_bytes: u64,
    rx_bytes: u64,
    last_handshake: u64,
    allowed_ips_count: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct SockAddrInet {
    addr: SOCKADDR_INET,
}

impl From<SOCKADDR_INET> for SockAddrInet {
    fn from(addr: SOCKADDR_INET) -> Self {
        Self { addr }
    }
}
impl PartialEq for SockAddrInet {
    fn eq(&self, other: &Self) -> bool {
        let self_addr = match try_sockaddr_to_socket_address(self.addr) {
            Ok(addr) => addr,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to convert socket address")
                );
                return true;
            }
        };
        let other_addr = match try_sockaddr_to_socket_address(other.addr) {
            Ok(addr) => addr,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to convert socket address")
                );
                return true;
            }
        };
        self_addr == other_addr
    }
}
impl Eq for SockAddrInet {}

impl fmt::Debug for SockAddrInet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("SockAddrInet");
        let self_addr = try_sockaddr_to_socket_address(self.addr)
            .map(|addr| addr.to_string())
            .unwrap_or("<unknown>".to_string());
        s.field("addr", &self_addr).finish()
    }
}

bitflags! {
    /// See `WIREGUARD_INTERFACE_FLAG` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
    struct WgInterfaceFlag: u32 {
        const HAS_PUBLIC_KEY = 0b00000001;
        const HAS_PRIVATE_KEY = 0b00000010;
        const HAS_LISTEN_PORT = 0b00000100;
        const REPLACE_PEERS = 0b00001000;
    }
}

/// See `WIREGUARD_INTERFACE` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C, align(8))]
struct WgInterface {
    flags: WgInterfaceFlag,
    listen_port: u16,
    private_key: [u8; WIREGUARD_KEY_LENGTH],
    public_key: [u8; WIREGUARD_KEY_LENGTH],
    peers_count: u32,
}

/// See `WIREGUARD_ADAPTER_LOG_STATE` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
enum WgAdapterState {
    Down = 0,
    Up = 1,
}


impl WgNtTunnel {
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        resource_dir: &Path,
    ) -> Result<Self> {
        let dll = load_wg_nt_dll(resource_dir)?;

        let logger_handle = LoggerHandle::new(dll.clone(), log_path)?;

        {
            if let Ok(device) = WgNtAdapter::open(dll.clone(), &*ADAPTER_POOL, &*ADAPTER_ALIAS) {
                device.delete().map_err(Error::DeleteExistingTunnelError)?;
            }
        }

        let (device, reboot_required) = WgNtAdapter::create(
            dll.clone(),
            &*ADAPTER_POOL,
            &*ADAPTER_ALIAS,
            Some(ADAPTER_GUID.clone()),
        )
        .map_err(Error::CreateTunnelDeviceError)?;

        if reboot_required {
            log::warn!("You may need to reboot to finish installing WireGuardNT");
        }

        let interface_luid = device.luid();
        let interface_name = match device.name() {
            Ok(name) => name.to_string_lossy(),
            Err(error) => {
                if let Err(error) = device.delete() {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to delete tunnel device")
                    );
                }
                return Err(Error::ObtainAliasError(error));
            }
        };

        let tunnel = WgNtTunnel {
            device: Some(device),
            interface_luid,
            interface_name,
            _logger_handle: logger_handle,
        };
        tunnel.configure(config)?;
        Ok(tunnel)
    }

    fn stop_tunnel(&mut self) -> Result<()> {
        if let Some(device) = self.device.take() {
            if let Err(error) = device.delete() {
                return Err(Error::DeleteTunnelDeviceError(error));
            }
        }
        Ok(())
    }

    fn configure(&self, config: &Config) -> Result<()> {
        let device = self.device.as_ref().unwrap();
        if let Err(error) = device.set_logging(WireGuardAdapterLogState::On) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to set log state on WireGuard interface")
            );
        }
        device.set_config(config)?;
        set_interface_mtu(&device.luid(), AF_INET as u16, u32::from(config.mtu))
            .map_err(Error::SetTunnelIpv4MtuError)?;
        if config.tunnel.addresses.iter().any(|addr| addr.is_ipv6()) {
            set_interface_mtu(&device.luid(), AF_INET6 as u16, u32::from(config.mtu))
                .map_err(Error::SetTunnelIpv6MtuError)?;
        }
        device
            .set_state(WgAdapterState::Up)
            .map_err(Error::EnableTunnelError)?;
        Ok(())
    }
}

impl Drop for WgNtTunnel {
    fn drop(&mut self) {
        if let Err(error) = self.stop_tunnel() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to stop WireGuardNT tunnel")
            );
        }
    }
}

lazy_static! {
    static ref LOG_CONTEXT: Mutex<Option<u32>> = Mutex::new(None);
}

struct LoggerHandle {
    dll: Arc<WgNtDll>,
    context: u32,
}

impl LoggerHandle {
    fn new(dll: Arc<WgNtDll>, log_path: Option<&Path>) -> Result<Self> {
        let context = logging::initialize_logging(log_path).map_err(Error::InitLoggingError)?;
        {
            *(LOG_CONTEXT.lock().unwrap()) = Some(context);
        }
        dll.set_logger(Some(Self::logging_callback));
        Ok(Self { dll, context })
    }

    extern "stdcall" fn logging_callback(level: LogLevel, _timestamp: u64, message: *const u16) {
        if message.is_null() {
            return;
        }
        let mut message = unsafe { U16CStr::from_ptr_str(message) }.to_string_lossy();
        message.push_str("\r\n");

        if let Some(context) = &*LOG_CONTEXT.lock().unwrap() {
            // Horribly broken, because callback does not provide a context
            logging::log(*context, level.into(), "wireguard-nt", &message);
        }
    }
}

impl Drop for LoggerHandle {
    fn drop(&mut self) {
        let mut ctx = LOG_CONTEXT.lock().unwrap();
        if *ctx == Some(self.context) {
            *ctx = None;
            self.dll.set_logger(None);
        }
        logging::clean_up_logging(self.context);
    }
}


struct WgNtAdapter {
    dll_handle: Arc<WgNtDll>,
    handle: RawHandle,
}

impl fmt::Debug for WgNtAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WgNtAdapter")
            .field("handle", &self.handle)
            .finish()
    }
}

unsafe impl Send for WgNtAdapter {}
unsafe impl Sync for WgNtAdapter {}

impl WgNtAdapter {
    fn open(dll_handle: Arc<WgNtDll>, pool: &U16CStr, name: &U16CStr) -> io::Result<Self> {
        let handle = dll_handle.open_adapter(pool, name)?;
        Ok(Self { dll_handle, handle })
    }

    fn create(
        dll_handle: Arc<WgNtDll>,
        pool: &U16CStr,
        name: &U16CStr,
        requested_guid: Option<GUID>,
    ) -> io::Result<(Self, RebootRequired)> {
        let (handle, restart_required) = dll_handle.create_adapter(pool, name, requested_guid)?;
        Ok((Self { dll_handle, handle }, restart_required))
    }

    fn delete(self) -> io::Result<RebootRequired> {
        unsafe { self.dll_handle.delete_adapter(self.handle) }
    }

    fn name(&self) -> io::Result<U16CString> {
        unsafe { self.dll_handle.get_adapter_name(self.handle) }
    }

    fn luid(&self) -> NET_LUID {
        unsafe { self.dll_handle.get_adapter_luid(self.handle) }
    }

    fn set_config(&self, config: &Config) -> Result<()> {
        let config_buffer = serialize_config(config)?;
        unsafe {
            self.dll_handle
                .set_config(self.handle, config_buffer.as_ptr(), config_buffer.len())
                .map_err(Error::SetWireGuardConfigError)
        }
    }

    fn get_config(&self) -> Result<(WgInterface, Vec<(WgPeer, Vec<WgAllowedIp>)>)> {
        unsafe {
            deserialize_config(
                &self
                    .dll_handle
                    .get_config(self.handle)
                    .map_err(Error::GetWireGuardConfigError)?,
            )
        }
    }

    fn set_state(&self, state: WgAdapterState) -> io::Result<()> {
        unsafe { self.dll_handle.set_adapter_state(self.handle, state) }
    }

    fn set_logging(&self, state: WireGuardAdapterLogState) -> io::Result<()> {
        unsafe { self.dll_handle.set_adapter_logging(self.handle, state) }
    }
}

impl Drop for WgNtAdapter {
    fn drop(&mut self) {
        unsafe { self.dll_handle.free_adapter(self.handle) };
    }
}

struct WgNtDll {
    handle: HINSTANCE,
    func_open: WireGuardOpenAdapterFn,
    func_create: WireGuardCreateAdapterFn,
    func_delete: WireGuardDeleteAdapterFn,
    func_free: WireGuardFreeAdapterFn,
    func_get_adapter_luid: WireGuardGetAdapterLuidFn,
    func_get_adapter_name: WireGuardGetAdapterNameFn,
    func_set_configuration: WireGuardSetConfigurationFn,
    func_get_configuration: WireGuardGetConfigurationFn,
    func_set_adapter_state: WireGuardSetStateFn,
    func_set_logger: WireGuardSetLoggerFn,
    func_set_adapter_logging: WireGuardSetAdapterLoggingFn,
}

unsafe impl Send for WgNtDll {}
unsafe impl Sync for WgNtDll {}

impl WgNtDll {
    pub fn new(resource_dir: &Path) -> io::Result<Self> {
        let wg_nt_dll: Vec<u16> = resource_dir
            .join("wireguard.dll")
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0u16))
            .collect();

        let handle = unsafe {
            LoadLibraryExW(
                wg_nt_dll.as_ptr(),
                ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            )
        };
        if handle == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }
        Self::new_inner(handle, Self::get_proc_address)
    }

    fn new_inner(
        handle: HMODULE,
        get_proc_fn: unsafe fn(HMODULE, &CStr) -> io::Result<FARPROC>,
    ) -> io::Result<Self> {
        Ok(WgNtDll {
            handle,
            func_open: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardOpenAdapter\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_create: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardCreateAdapter\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_delete: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardDeleteAdapter\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_free: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardFreeAdapter\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_get_adapter_luid: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetAdapterLUID\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_get_adapter_name: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetAdapterName\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_set_configuration: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardSetConfiguration\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_get_configuration: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetConfiguration\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_set_adapter_state: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardSetAdapterState\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_set_logger: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardSetLogger\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_set_adapter_logging: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardSetAdapterLogging\0").unwrap(),
                )?) as *const _ as *const _)
            },
        })
    }

    unsafe fn get_proc_address(handle: HMODULE, name: &CStr) -> io::Result<FARPROC> {
        let handle = GetProcAddress(handle, name.as_ptr());
        if handle == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }

    pub fn open_adapter(&self, pool: &U16CStr, name: &U16CStr) -> io::Result<RawHandle> {
        let handle = unsafe { (self.func_open)(pool.as_ptr(), name.as_ptr()) };
        if handle == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }

    pub fn create_adapter(
        &self,
        pool: &U16CStr,
        name: &U16CStr,
        requested_guid: Option<GUID>,
    ) -> io::Result<(RawHandle, RebootRequired)> {
        let guid_ptr = match requested_guid.as_ref() {
            Some(guid) => guid as *const _,
            None => ptr::null_mut(),
        };
        let mut reboot_required = 0;
        let handle = unsafe {
            (self.func_create)(pool.as_ptr(), name.as_ptr(), guid_ptr, &mut reboot_required)
        };
        if handle == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }
        Ok((handle, reboot_required != 0))
    }

    pub unsafe fn delete_adapter(&self, adapter: RawHandle) -> io::Result<RebootRequired> {
        let mut reboot_required = 0;
        let result = (self.func_delete)(adapter, &mut reboot_required);
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(reboot_required != 0)
    }

    pub unsafe fn free_adapter(&self, adapter: RawHandle) {
        (self.func_free)(adapter);
    }

    pub unsafe fn get_adapter_name(&self, adapter: RawHandle) -> io::Result<U16CString> {
        let mut alias_buffer = vec![0u16; MAX_ADAPTER_NAME];
        let result = (self.func_get_adapter_name)(adapter, alias_buffer.as_mut_ptr());
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(U16CString::from_vec_with_nul(alias_buffer)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing null terminator"))?)
    }

    pub unsafe fn get_adapter_luid(&self, adapter: RawHandle) -> NET_LUID {
        let mut luid = mem::MaybeUninit::<NET_LUID>::zeroed();
        (self.func_get_adapter_luid)(adapter, luid.as_mut_ptr());
        luid.assume_init()
    }

    pub unsafe fn set_config(
        &self,
        adapter: RawHandle,
        config: *const u8,
        config_size: usize,
    ) -> io::Result<()> {
        let result = (self.func_set_configuration)(adapter, config, config_size as u32);
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub unsafe fn get_config(&self, adapter: RawHandle) -> io::Result<Vec<u8>> {
        let mut config_size = 0;
        let mut config = vec![];
        loop {
            let result =
                (self.func_get_configuration)(adapter, config.as_mut_ptr(), &mut config_size);
            if result == 0 {
                let last_error = io::Error::last_os_error();
                if last_error.raw_os_error() != Some(ERROR_MORE_DATA as i32) {
                    break Err(last_error);
                }
                config.resize(config_size as usize, 0);
            } else {
                break Ok(config);
            }
        }
    }

    pub unsafe fn set_adapter_state(
        &self,
        adapter: RawHandle,
        state: WgAdapterState,
    ) -> io::Result<()> {
        let result = (self.func_set_adapter_state)(adapter, state);
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn set_logger(&self, cb: Option<WireGuardLoggerCb>) {
        (self.func_set_logger)(cb);
    }

    pub unsafe fn set_adapter_logging(
        &self,
        adapter: RawHandle,
        state: WireGuardAdapterLogState,
    ) -> io::Result<()> {
        if (self.func_set_adapter_logging)(adapter, state) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for WgNtDll {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.handle) };
    }
}

fn load_wg_nt_dll(resource_dir: &Path) -> Result<Arc<WgNtDll>> {
    let mut dll = (*WG_NT_DLL).lock().expect("WireGuardNT mutex poisoned");
    match &*dll {
        Some(dll) => Ok(dll.clone()),
        None => {
            let new_dll = Arc::new(WgNtDll::new(resource_dir).map_err(Error::DllError)?);
            *dll = Some(new_dll.clone());
            Ok(new_dll)
        }
    }
}

fn serialize_config(config: &Config) -> Result<Vec<u8>> {
    let mut buffer = vec![];

    let header = WgInterface {
        flags: WgInterfaceFlag::HAS_PRIVATE_KEY | WgInterfaceFlag::REPLACE_PEERS,
        listen_port: 0,
        private_key: config.tunnel.private_key.to_bytes(),
        public_key: [0u8; WIREGUARD_KEY_LENGTH],
        peers_count: config.peers.len() as u32,
    };

    buffer.extend_from_slice(unsafe { as_u8_slice(&header) });

    for peer in &config.peers {
        let wg_peer = WgPeer {
            flags: WgPeerFlag::HAS_PUBLIC_KEY | WgPeerFlag::HAS_ENDPOINT,
            reserved: 0,
            public_key: peer.public_key.as_bytes().clone(),
            preshared_key: [0u8; WIREGUARD_KEY_LENGTH],
            persistent_keepalive: 0,
            endpoint: convert_socket_address(peer.endpoint).into(),
            tx_bytes: 0,
            rx_bytes: 0,
            last_handshake: 0,
            allowed_ips_count: peer.allowed_ips.len() as u32,
        };

        buffer.extend_from_slice(unsafe { as_u8_slice(&wg_peer) });

        for allowed_ip in &peer.allowed_ips {
            let address_family = match allowed_ip {
                IpNetwork::V4(_) => AF_INET as u16,
                IpNetwork::V6(_) => AF_INET6 as u16,
            };
            let address = match allowed_ip {
                IpNetwork::V4(v4_network) => WgIpAddr {
                    v4: convert_v4_address(v4_network.ip()),
                },
                IpNetwork::V6(v6_network) => WgIpAddr {
                    v6: convert_v6_address(v6_network.ip()),
                },
            };

            let wg_allowed_ip =
                WgAllowedIp::new(address, address_family, allowed_ip.prefix() as u8)?;

            buffer.extend_from_slice(unsafe { as_u8_slice(&wg_allowed_ip) });
        }
    }

    Ok(buffer)
}

unsafe fn deserialize_config(
    config: &[u8],
) -> Result<(WgInterface, Vec<(WgPeer, Vec<WgAllowedIp>)>)> {
    if config.len() < mem::size_of::<WgInterface>() {
        return Err(Error::InvalidConfigData);
    }
    let (head, mut tail) = config.split_at(mem::size_of::<WgInterface>());
    let interface: WgInterface = *(head.as_ptr() as *const WgInterface);

    let mut peers = vec![];
    for _ in 0..interface.peers_count {
        if tail.len() < mem::size_of::<WgPeer>() {
            return Err(Error::InvalidConfigData);
        }
        let (peer_data, new_tail) = tail.split_at(mem::size_of::<WgPeer>());
        let peer: WgPeer = *(peer_data.as_ptr() as *const WgPeer);
        tail = new_tail;

        if let Err(error) = windows::try_socketaddr_from_inet_sockaddr(peer.endpoint.addr) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Received invalid endpoint address")
            );
            return Err(Error::InvalidConfigData);
        }

        let mut allowed_ips = vec![];

        for _ in 0..peer.allowed_ips_count {
            if tail.len() < mem::size_of::<WgAllowedIp>() {
                return Err(Error::InvalidConfigData);
            }
            let (allowed_ip_data, new_tail) = tail.split_at(mem::size_of::<WgAllowedIp>());
            let allowed_ip: WgAllowedIp = *(allowed_ip_data.as_ptr() as *const WgAllowedIp);
            if let Err(error) = WgAllowedIp::validate(
                &allowed_ip.address,
                allowed_ip.address_family,
                allowed_ip.cidr,
            ) {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Received invalid allowed IP")
                );
                return Err(Error::InvalidConfigData);
            }
            tail = new_tail;
            allowed_ips.push(allowed_ip);
        }

        peers.push((peer, allowed_ips));
    }

    if tail.len() > 0 {
        return Err(Error::InvalidConfigData);
    }

    Ok((interface, peers))
}

fn convert_v4_address(addr: Ipv4Addr) -> IN_ADDR {
    let mut in_addr: IN_ADDR = unsafe { mem::zeroed() };
    let addr_octets = addr.octets();
    unsafe {
        ptr::copy_nonoverlapping(
            &addr_octets as *const _,
            in_addr.S_un.S_addr_mut() as *mut _ as *mut u8,
            addr_octets.len(),
        );
    }
    in_addr
}

fn convert_v6_address(addr: Ipv6Addr) -> IN6_ADDR {
    let mut in_addr: IN6_ADDR = unsafe { mem::zeroed() };
    let addr_octets = addr.octets();
    unsafe {
        ptr::copy_nonoverlapping(
            &addr_octets as *const _,
            in_addr.u.Byte_mut() as *mut _,
            addr_octets.len(),
        );
    }
    in_addr
}

fn convert_socket_address(addr: SocketAddr) -> SOCKADDR_INET {
    let mut sockaddr: SOCKADDR_INET = unsafe { mem::zeroed() };

    match addr {
        SocketAddr::V4(v4_addr) => {
            unsafe {
                *sockaddr.si_family_mut() = AF_INET as u16;
            }

            let mut v4sockaddr = unsafe { sockaddr.Ipv4_mut() };
            v4sockaddr.sin_family = AF_INET as u16;
            v4sockaddr.sin_port = v4_addr.port().to_be();
            v4sockaddr.sin_addr = convert_v4_address(*v4_addr.ip());
        }
        SocketAddr::V6(v6_addr) => {
            unsafe {
                *sockaddr.si_family_mut() = AF_INET6 as u16;
            }

            let mut v6sockaddr = unsafe { sockaddr.Ipv6_mut() };
            v6sockaddr.sin6_family = AF_INET6 as u16;
            v6sockaddr.sin6_port = v6_addr.port().to_be();
            v6sockaddr.sin6_addr = convert_v6_address(*v6_addr.ip());
            v6sockaddr.sin6_flowinfo = v6_addr.flowinfo();
            *unsafe { v6sockaddr.u.sin6_scope_id_mut() } = v6_addr.scope_id();
        }
    }

    sockaddr
}

fn inaddr_to_ipaddr(addr: IN_ADDR) -> Ipv4Addr {
    Ipv4Addr::from(unsafe { *(addr.S_un.S_addr()) }.to_be())
}

fn in6addr_to_ipaddr(addr: IN6_ADDR) -> Ipv6Addr {
    Ipv6Addr::from(*unsafe { addr.u.Byte() })
}

fn try_sockaddr_to_socket_address(addr: SOCKADDR_INET) -> Result<SocketAddr> {
    unsafe {
        match *addr.si_family() as i32 {
            AF_INET => Ok(SocketAddr::V4(SocketAddrV4::new(
                inaddr_to_ipaddr(addr.Ipv4().sin_addr),
                u16::from_be(addr.Ipv4().sin_port),
            ))),
            AF_INET6 => Ok(SocketAddr::V6(SocketAddrV6::new(
                in6addr_to_ipaddr(addr.Ipv6().sin6_addr),
                u16::from_be(addr.Ipv6().sin6_port),
                addr.Ipv6().sin6_flowinfo,
                *addr.Ipv6().u.sin6_scope_id(),
            ))),
            family => Err(Error::UnknownAddressFamily(family)),
        }
    }
}

fn set_interface_mtu(luid: &NET_LUID, family: u16, mtu: u32) -> io::Result<()> {
    let family = crate::tunnel::windows::AddressFamily::try_from_af_family(family)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let mut iface = crate::tunnel::windows::get_ip_interface_entry(family, luid)?;
    iface.SitePrefixLength = 0;
    iface.NlMtu = mtu;
    crate::tunnel::windows::set_ip_interface_entry(&iface)
}

impl Tunnel for WgNtTunnel {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn get_interface_luid(&self) -> u64 {
        self.interface_luid.Value
    }

    fn get_tunnel_stats(&self) -> std::result::Result<StatsMap, super::TunnelError> {
        if let Some(ref device) = self.device {
            let mut map = StatsMap::new();
            let (_interface, peers) = device.get_config().map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to obtain wg-nt tunnel config")
                );
                super::TunnelError::StatsError(super::stats::Error::NoTunnelConfig)
            })?;
            for (peer, _allowed_ips) in &peers {
                map.insert(
                    peer.public_key,
                    Stats {
                        tx_bytes: peer.tx_bytes,
                        rx_bytes: peer.rx_bytes,
                    },
                );
            }
            Ok(map)
        } else {
            Err(super::TunnelError::StatsError(
                super::stats::Error::NoTunnelDevice,
            ))
        }
    }

    fn stop(mut self: Box<Self>) -> std::result::Result<(), super::TunnelError> {
        if let Err(error) = self.stop_tunnel() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to stop WireGuardNT tunnel")
            );
            Err(super::TunnelError::StopWireguardError { status: 0 })
        } else {
            Ok(())
        }
    }
}

unsafe fn as_u8_slice<T: Sized>(object: &T) -> &[u8] {
    std::slice::from_raw_parts(object as *const _ as *const _, mem::size_of::<T>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use talpid_types::net::{wireguard, TransportProtocol};

    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    #[repr(C)]
    struct Interface {
        interface: WgInterface,
        p0: WgPeer,
        p0_allowed_ip_0: WgAllowedIp,
    }

    lazy_static! {
        static ref WG_PRIVATE_KEY: wireguard::PrivateKey = wireguard::PrivateKey::new_from_random();
        static ref WG_PUBLIC_KEY: wireguard::PublicKey =
            wireguard::PrivateKey::new_from_random().public_key();
        static ref WG_CONFIG: Config = {
            Config {
                tunnel: wireguard::TunnelConfig {
                    private_key: WG_PRIVATE_KEY.clone(),
                    addresses: vec![],
                },
                peers: vec![wireguard::PeerConfig {
                    public_key: WG_PUBLIC_KEY.clone(),
                    allowed_ips: vec!["1.3.3.0/24".parse().unwrap()],
                    endpoint: "1.2.3.4:1234".parse().unwrap(),
                    protocol: TransportProtocol::Udp,
                }],
                ipv4_gateway: "0.0.0.0".parse().unwrap(),
                ipv6_gateway: None,
                mtu: 0,
                use_wireguard_nt: true,
            }
        };
        static ref WG_STRUCT_CONFIG: Interface = Interface {
            interface: WgInterface {
                flags: WgInterfaceFlag::HAS_PRIVATE_KEY | WgInterfaceFlag::REPLACE_PEERS,
                listen_port: 0,
                private_key: WG_PRIVATE_KEY.to_bytes(),
                public_key: [0; WIREGUARD_KEY_LENGTH],
                peers_count: 1,
            },
            p0: WgPeer {
                flags: WgPeerFlag::HAS_PUBLIC_KEY | WgPeerFlag::HAS_ENDPOINT,
                reserved: 0,
                public_key: WG_PUBLIC_KEY.as_bytes().clone(),
                preshared_key: [0; WIREGUARD_KEY_LENGTH],
                persistent_keepalive: 0,
                endpoint: convert_socket_address("1.2.3.4:1234".parse().unwrap()).into(),
                tx_bytes: 0,
                rx_bytes: 0,
                last_handshake: 0,
                allowed_ips_count: 1,
            },
            p0_allowed_ip_0: WgAllowedIp {
                address: WgIpAddr {
                    v4: convert_v4_address("1.3.3.0".parse().unwrap()),
                },
                address_family: AF_INET as u16,
                cidr: 24,
            },
        };
    }

    fn get_proc_fn(_handle: HMODULE, _symbol: &CStr) -> io::Result<FARPROC> {
        Ok(std::ptr::null_mut())
    }

    #[test]
    fn test_dll_imports() {
        WgNtDll::new_inner(ptr::null_mut(), get_proc_fn).unwrap();
    }

    #[test]
    fn test_sockaddr_v4() {
        let addr_v4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 1234));
        assert_eq!(
            addr_v4,
            try_sockaddr_to_socket_address(convert_socket_address(addr_v4)).unwrap()
        );
    }

    #[test]
    fn test_sockaddr_v6() {
        let addr_v6 = SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8),
            1234,
            0xa,
            0xb,
        ));
        assert_eq!(
            addr_v6,
            try_sockaddr_to_socket_address(convert_socket_address(addr_v6)).unwrap()
        );
    }

    #[test]
    fn test_config_serialization() {
        let serialized_data = serialize_config(&*WG_CONFIG).unwrap();
        assert_eq!(mem::size_of::<Interface>(), serialized_data.len());
        let serialized_iface = &unsafe { *(serialized_data.as_ptr() as *const Interface) };
        assert_eq!(&*WG_STRUCT_CONFIG, serialized_iface);
    }

    #[test]
    fn test_config_deserialization() {
        let (iface, peers) =
            unsafe { deserialize_config(as_u8_slice(&*WG_STRUCT_CONFIG)) }.unwrap();
        assert_eq!(iface, WG_STRUCT_CONFIG.interface);
        assert_eq!(peers.len(), 1);
        let (peer, allowed_ips) = &peers[0];
        assert_eq!(peer, &WG_STRUCT_CONFIG.p0);
        assert_eq!(allowed_ips.len(), 1);
        assert_eq!(allowed_ips[0], WG_STRUCT_CONFIG.p0_allowed_ip_0);
    }

    #[test]
    fn test_wg_allowed_ip_v4() {
        // Valid: /32 prefix
        let address_family = AF_INET as u16;
        let address = WgIpAddr {
            v4: convert_v4_address("127.0.0.1".parse().unwrap()),
        };
        let cidr = 32;
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid host bits
        let cidr = 24;
        let address = WgIpAddr {
            v4: convert_v4_address("0.0.0.1".parse().unwrap()),
        };
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());

        // Valid host bits
        let cidr = 24;
        let address = WgIpAddr {
            v4: convert_v4_address("255.255.255.0".parse().unwrap()),
        };
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // 0.0.0.0/0
        let cidr = 0;
        let address = WgIpAddr {
            v4: convert_v4_address("0.0.0.0".parse().unwrap()),
        };
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid CIDR
        let cidr = 33;
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());
    }

    #[test]
    fn test_wg_allowed_ip_v6() {
        // Valid: /128 prefix
        let address_family = AF_INET6 as u16;
        let address = WgIpAddr {
            v6: convert_v6_address("::1".parse().unwrap()),
        };
        let cidr = 128;
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid host bits
        let cidr = 127;
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());

        // Valid host bits
        let address = WgIpAddr {
            v6: convert_v6_address("ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe".parse().unwrap()),
        };
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // ::/0
        let cidr = 0;
        let address = WgIpAddr {
            v6: convert_v6_address("::".parse().unwrap()),
        };
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid CIDR
        let cidr = 129;
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());
    }
}
