use super::{
    config::Config,
    logging,
    stats::{Stats, StatsMap},
    Tunnel,
};
use bitflags::bitflags;
use futures::SinkExt;
use ipnetwork::IpNetwork;
use lazy_static::lazy_static;
use std::{
    ffi::CStr,
    fmt,
    future::Future,
    io, mem,
    mem::MaybeUninit,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    os::windows::io::RawHandle,
    path::Path,
    pin::Pin,
    ptr,
    sync::{Arc, Mutex},
};
use talpid_types::{BoxedError, ErrorExt};
use talpid_windows_net as net;
use widestring::{U16CStr, U16CString};
use windows_sys::{
    core::GUID,
    Win32::{
        Foundation::{BOOL, ERROR_MORE_DATA, HINSTANCE},
        NetworkManagement::Ndis::NET_LUID_LH,
        Networking::WinSock::{
            ADDRESS_FAMILY, AF_INET, AF_INET6, IN6_ADDR, IN_ADDR, SOCKADDR_INET,
        },
        System::LibraryLoader::{
            FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH,
        },
    },
};

lazy_static! {
    static ref WG_NT_DLL: Mutex<Option<Arc<WgNtDll>>> = Mutex::new(None);
    static ref ADAPTER_TYPE: U16CString = U16CString::from_str("Mullvad").unwrap();
    static ref ADAPTER_ALIAS: U16CString = U16CString::from_str("Mullvad").unwrap();
}

const ADAPTER_GUID: GUID = GUID {
    data1: 0x514a3988,
    data2: 0x9716,
    data3: 0x43d5,
    data4: [0x8b, 0x05, 0x31, 0xda, 0x25, 0xa0, 0x44, 0xa9],
};

type WireGuardCreateAdapterFn = unsafe extern "stdcall" fn(
    name: *const u16,
    tunnel_type: *const u16,
    requested_guid: *const GUID,
) -> RawHandle;
type WireGuardCloseAdapterFn = unsafe extern "stdcall" fn(adapter: RawHandle);
type WireGuardGetAdapterLuidFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, luid: *mut NET_LUID_LH);
type WireGuardSetConfigurationFn = unsafe extern "stdcall" fn(
    adapter: RawHandle,
    config: *const MaybeUninit<u8>,
    bytes: u32,
) -> BOOL;
type WireGuardGetConfigurationFn = unsafe extern "stdcall" fn(
    adapter: RawHandle,
    config: *const MaybeUninit<u8>,
    bytes: *mut u32,
) -> BOOL;
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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to load WireGuardNT
    #[error(display = "Failed to load mullvad-wireguard.dll")]
    DllError(#[error(source)] io::Error),

    /// Failed to create tunnel interface
    #[error(display = "Failed to create WireGuard device")]
    CreateTunnelDeviceError(#[error(source)] io::Error),

    /// Failed to obtain tunnel interface alias
    #[error(display = "Failed to obtain interface name")]
    ObtainAliasError(#[error(source)] io::Error),

    /// Failed to get WireGuard tunnel config for device
    #[error(display = "Failed to get tunnel WireGuard config")]
    GetWireGuardConfigError(#[error(source)] io::Error),

    /// Failed to set WireGuard tunnel config on device
    #[error(display = "Failed to set tunnel WireGuard config")]
    SetWireGuardConfigError(#[error(source)] io::Error),

    /// Error listening to tunnel IP interfaces
    #[error(display = "Failed to wait on tunnel IP interfaces")]
    IpInterfacesError(#[error(source)] io::Error),

    /// Failed to set MTU and metric on tunnel device
    #[error(display = "Failed to set tunnel interface MTU")]
    SetTunnelMtuError(#[error(source)] io::Error),

    /// Failed to set the tunnel state to up
    #[error(display = "Failed to enable the tunnel adapter")]
    EnableTunnelError(#[error(source)] io::Error),

    /// Unknown address family
    #[error(display = "Unknown address family: {}", _0)]
    UnknownAddressFamily(u32),

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
    device: Arc<Mutex<Option<WgNtAdapter>>>,
    interface_name: String,
    setup_handle: tokio::task::JoinHandle<()>,
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

impl From<IpAddr> for WgIpAddr {
    fn from(address: IpAddr) -> Self {
        match address {
            IpAddr::V4(addr) => WgIpAddr::from(addr),
            IpAddr::V6(addr) => WgIpAddr::from(addr),
        }
    }
}

impl From<Ipv6Addr> for WgIpAddr {
    fn from(address: Ipv6Addr) -> Self {
        Self {
            v6: net::in6addr_from_ipaddr(address),
        }
    }
}

impl From<Ipv4Addr> for WgIpAddr {
    fn from(address: Ipv4Addr) -> Self {
        Self {
            v4: net::inaddr_from_ipaddr(address),
        }
    }
}

/// See `WIREGUARD_ALLOWED_IP` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Clone, Copy)]
#[repr(C, align(8))]
struct WgAllowedIp {
    address: WgIpAddr,
    address_family: u16,
    cidr: u8,
}

impl WgAllowedIp {
    fn new(address: WgIpAddr, address_family: ADDRESS_FAMILY, cidr: u8) -> Result<Self> {
        Self::validate(&address, address_family, cidr)?;
        Ok(Self {
            address,
            address_family: address_family as u16,
            cidr,
        })
    }

    fn validate(address: &WgIpAddr, address_family: ADDRESS_FAMILY, cidr: u8) -> Result<()> {
        match address_family {
            AF_INET => {
                if cidr > 32 {
                    return Err(Error::InvalidAllowedIpCidr);
                }
                let host_mask = u32::MAX.checked_shr(u32::from(cidr)).unwrap_or(0);
                if host_mask & unsafe { address.v4.S_un.S_addr }.to_be() != 0 {
                    return Err(Error::InvalidAllowedIpBits);
                }
            }
            AF_INET6 => {
                if cidr > 128 {
                    return Err(Error::InvalidAllowedIpCidr);
                }
                let mut host_mask = u128::MAX.checked_shr(u32::from(cidr)).unwrap_or(0);
                let bytes = unsafe { address.v6.u.Byte };
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
        match self.address_family as u32 {
            AF_INET => {
                net::ipaddr_from_inaddr(unsafe { self.address.v4 })
                    == net::ipaddr_from_inaddr(unsafe { other.address.v4 })
            }
            AF_INET6 => {
                net::ipaddr_from_in6addr(unsafe { self.address.v6 })
                    == net::ipaddr_from_in6addr(unsafe { other.address.v6 })
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
        match self.address_family as u32 {
            AF_INET => s.field(
                "address",
                &net::ipaddr_from_inaddr(unsafe { self.address.v4 }),
            ),
            AF_INET6 => s.field(
                "address",
                &net::ipaddr_from_in6addr(unsafe { self.address.v6 }),
            ),
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
        let self_addr = match net::try_socketaddr_from_inet_sockaddr(self.addr) {
            Ok(addr) => addr,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to convert socket address")
                );
                return true;
            }
        };
        let other_addr = match net::try_socketaddr_from_inet_sockaddr(other.addr) {
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
        let self_addr = net::try_socketaddr_from_inet_sockaddr(self.addr)
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
        done_tx: futures::channel::mpsc::Sender<std::result::Result<(), BoxedError>>,
    ) -> std::result::Result<Self, super::TunnelError> {
        Self::start_tunnel_inner(config, log_path, resource_dir, done_tx).map_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to setup WireGuardNT tunnel")
            );

            match error {
                Error::CreateTunnelDeviceError(_) => {
                    super::TunnelError::RecoverableStartWireguardError
                }
                _ => super::TunnelError::FatalStartWireguardError,
            }
        })
    }

    fn start_tunnel_inner(
        config: &Config,
        log_path: Option<&Path>,
        resource_dir: &Path,
        mut done_tx: futures::channel::mpsc::Sender<std::result::Result<(), BoxedError>>,
    ) -> Result<Self> {
        let dll = load_wg_nt_dll(resource_dir)?;
        let logger_handle = LoggerHandle::new(dll.clone(), log_path)?;
        let device = WgNtAdapter::create(
            dll.clone(),
            &*ADAPTER_ALIAS,
            &*ADAPTER_TYPE,
            Some(ADAPTER_GUID.clone()),
        )
        .map_err(Error::CreateTunnelDeviceError)?;

        let interface_name = device
            .name()
            .map_err(Error::ObtainAliasError)?
            .to_string_lossy();

        if let Err(error) = device.set_logging(WireGuardAdapterLogState::On) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to set log state on WireGuard interface")
            );
        }
        device.set_config(config)?;
        let device = Arc::new(Mutex::new(Some(device)));

        let setup_future = setup_ip_listener(
            device.clone(),
            u32::from(config.mtu),
            config.tunnel.addresses.iter().any(|addr| addr.is_ipv6()),
        );
        let setup_handle = tokio::spawn(async move {
            let _ = done_tx
                .send(setup_future.await.map_err(BoxedError::new))
                .await;
        });

        Ok(WgNtTunnel {
            device,
            interface_name,
            setup_handle,
            _logger_handle: logger_handle,
        })
    }

    fn stop_tunnel(&mut self) {
        self.setup_handle.abort();
        let _ = self.device.lock().unwrap().take();
    }
}

async fn setup_ip_listener(
    device: Arc<Mutex<Option<WgNtAdapter>>>,
    mtu: u32,
    has_ipv6: bool,
) -> Result<()> {
    let luid = { device.lock().unwrap().as_ref().unwrap().luid() };
    let luid = NET_LUID_LH {
        Value: unsafe { luid.Value },
    };

    log::debug!("Waiting for tunnel IP interfaces to arrive");
    net::wait_for_interfaces(luid, true, has_ipv6)
        .await
        .map_err(Error::IpInterfacesError)?;
    log::debug!("Waiting for tunnel IP interfaces: Done");

    talpid_tunnel::network_interface::initialize_interfaces(luid, Some(mtu))
        .map_err(Error::SetTunnelMtuError)?;

    if let Some(device) = &*device.lock().unwrap() {
        device
            .set_state(WgAdapterState::Up)
            .map_err(Error::EnableTunnelError)
    } else {
        Ok(())
    }
}

impl Drop for WgNtTunnel {
    fn drop(&mut self) {
        self.stop_tunnel();
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
    fn create(
        dll_handle: Arc<WgNtDll>,
        name: &U16CStr,
        tunnel_type: &U16CStr,
        requested_guid: Option<GUID>,
    ) -> io::Result<Self> {
        let handle = dll_handle.create_adapter(name, tunnel_type, requested_guid)?;
        Ok(Self { dll_handle, handle })
    }

    fn name(&self) -> io::Result<U16CString> {
        net::alias_from_luid(&self.luid()).and_then(|alias| {
            U16CString::from_os_str(alias)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "unexpected null char"))
        })
    }

    fn luid(&self) -> NET_LUID_LH {
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
        unsafe { self.dll_handle.close_adapter(self.handle) };
    }
}

struct WgNtDll {
    handle: HINSTANCE,
    func_create: WireGuardCreateAdapterFn,
    func_close: WireGuardCloseAdapterFn,
    func_get_adapter_luid: WireGuardGetAdapterLuidFn,
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
        let wg_nt_dll =
            U16CString::from_os_str_truncate(resource_dir.join("mullvad-wireguard.dll"));

        let handle =
            unsafe { LoadLibraryExW(wg_nt_dll.as_ptr(), 0, LOAD_WITH_ALTERED_SEARCH_PATH) };
        if handle == 0 {
            return Err(io::Error::last_os_error());
        }
        Self::new_inner(handle, Self::get_proc_address)
    }

    fn new_inner(
        handle: HINSTANCE,
        get_proc_fn: unsafe fn(
            HINSTANCE,
            &CStr,
        ) -> io::Result<unsafe extern "system" fn() -> isize>,
    ) -> io::Result<Self> {
        Ok(WgNtDll {
            handle,
            func_create: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardCreateAdapter\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_close: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardCloseAdapter\0").unwrap(),
                )?) as *const _ as *const _)
            },
            func_get_adapter_luid: unsafe {
                *((&get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetAdapterLUID\0").unwrap(),
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

    unsafe fn get_proc_address(
        handle: HINSTANCE,
        name: &CStr,
    ) -> io::Result<unsafe extern "system" fn() -> isize> {
        let handle = GetProcAddress(handle, name.as_ptr() as *const u8);
        handle.ok_or(io::Error::last_os_error())
    }

    pub fn create_adapter(
        &self,
        name: &U16CStr,
        tunnel_type: &U16CStr,
        requested_guid: Option<GUID>,
    ) -> io::Result<RawHandle> {
        let guid_ptr = match requested_guid.as_ref() {
            Some(guid) => guid as *const _,
            None => ptr::null_mut(),
        };
        let handle = unsafe { (self.func_create)(name.as_ptr(), tunnel_type.as_ptr(), guid_ptr) };
        if handle == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }

    pub unsafe fn close_adapter(&self, adapter: RawHandle) {
        (self.func_close)(adapter);
    }

    pub unsafe fn get_adapter_luid(&self, adapter: RawHandle) -> NET_LUID_LH {
        let mut luid = mem::MaybeUninit::<NET_LUID_LH>::zeroed();
        (self.func_get_adapter_luid)(adapter, luid.as_mut_ptr());
        luid.assume_init()
    }

    pub unsafe fn set_config(
        &self,
        adapter: RawHandle,
        config: *const MaybeUninit<u8>,
        config_size: usize,
    ) -> io::Result<()> {
        let result = (self.func_set_configuration)(adapter, config, config_size as u32);
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub unsafe fn get_config(&self, adapter: RawHandle) -> io::Result<Vec<MaybeUninit<u8>>> {
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
                config.resize(config_size as usize, MaybeUninit::new(0u8));
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

fn serialize_config(config: &Config) -> Result<Vec<MaybeUninit<u8>>> {
    let mut buffer = vec![];

    let header = WgInterface {
        flags: WgInterfaceFlag::HAS_PRIVATE_KEY | WgInterfaceFlag::REPLACE_PEERS,
        listen_port: 0,
        private_key: config.tunnel.private_key.to_bytes(),
        public_key: [0u8; WIREGUARD_KEY_LENGTH],
        peers_count: config.peers.len() as u32,
    };

    buffer.extend(as_uninit_byte_slice(&header));

    for peer in &config.peers {
        let flags = if peer.psk.is_some() {
            WgPeerFlag::HAS_PRESHARED_KEY | WgPeerFlag::HAS_PUBLIC_KEY | WgPeerFlag::HAS_ENDPOINT
        } else {
            WgPeerFlag::HAS_PUBLIC_KEY | WgPeerFlag::HAS_ENDPOINT
        };
        let wg_peer = WgPeer {
            flags,
            reserved: 0,
            public_key: peer.public_key.as_bytes().clone(),
            preshared_key: peer
                .psk
                .as_ref()
                .map(|psk| psk.as_bytes().clone())
                .unwrap_or([0u8; WIREGUARD_KEY_LENGTH]),
            persistent_keepalive: 0,
            endpoint: net::inet_sockaddr_from_socketaddr(peer.endpoint).into(),
            tx_bytes: 0,
            rx_bytes: 0,
            last_handshake: 0,
            allowed_ips_count: peer.allowed_ips.len() as u32,
        };

        buffer.extend(as_uninit_byte_slice(&wg_peer));

        for allowed_ip in &peer.allowed_ips {
            let address_family = match allowed_ip {
                IpNetwork::V4(_) => AF_INET,
                IpNetwork::V6(_) => AF_INET6,
            };
            let address = match allowed_ip {
                IpNetwork::V4(v4_network) => WgIpAddr::from(v4_network.ip()),
                IpNetwork::V6(v6_network) => WgIpAddr::from(v6_network.ip()),
            };

            let wg_allowed_ip =
                WgAllowedIp::new(address, address_family, allowed_ip.prefix() as u8)?;

            buffer.extend(as_uninit_byte_slice(&wg_allowed_ip));
        }
    }

    Ok(buffer)
}

unsafe fn deserialize_config(
    config: &[MaybeUninit<u8>],
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

        if let Err(error) = net::try_socketaddr_from_inet_sockaddr(peer.endpoint.addr) {
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
                u32::from(allowed_ip.address_family),
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

impl Tunnel for WgNtTunnel {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn get_tunnel_stats(&self) -> std::result::Result<StatsMap, super::TunnelError> {
        if let Some(ref device) = &*self.device.lock().unwrap() {
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
        self.stop_tunnel();
        Ok(())
    }

    fn set_config(
        &self,
        config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), super::TunnelError>> + Send>> {
        let device = self.device.clone();
        Box::pin(async move {
            let guard = device.lock().unwrap();
            let device = guard.as_ref().ok_or(super::TunnelError::SetConfigError)?;
            device.set_config(&config).map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to set wg-nt tunnel config")
                );
                super::TunnelError::SetConfigError
            })
        })
    }
}

pub fn as_uninit_byte_slice<T: Copy + Sized>(value: &T) -> &[mem::MaybeUninit<u8>] {
    unsafe { std::slice::from_raw_parts(value as *const _ as *const _, mem::size_of::<T>()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use talpid_types::net::wireguard;

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
                    psk: None,
                }],
                ipv4_gateway: "0.0.0.0".parse().unwrap(),
                ipv6_gateway: None,
                mtu: 0,
                use_wireguard_nt: true,
                obfuscator_config: None,
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
                endpoint: talpid_windows_net::inet_sockaddr_from_socketaddr(
                    "1.2.3.4:1234".parse().unwrap()
                )
                .into(),
                tx_bytes: 0,
                rx_bytes: 0,
                last_handshake: 0,
                allowed_ips_count: 1,
            },
            p0_allowed_ip_0: WgAllowedIp {
                address: WgIpAddr::from("1.3.3.0".parse::<Ipv4Addr>().unwrap()),
                address_family: AF_INET as u16,
                cidr: 24,
            },
        };
    }

    fn get_proc_fn(
        _handle: HINSTANCE,
        _symbol: &CStr,
    ) -> io::Result<unsafe extern "system" fn() -> isize> {
        Ok(null_fn)
    }

    #[test]
    fn test_dll_imports() {
        WgNtDll::new_inner(0, get_proc_fn).unwrap();
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
        let config_buffer = as_uninit_byte_slice(&*WG_STRUCT_CONFIG);
        let (iface, peers) = unsafe { deserialize_config(config_buffer) }.unwrap();
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
        let address_family = AF_INET;
        let address = WgIpAddr::from("127.0.0.1".parse::<Ipv4Addr>().unwrap());
        let cidr = 32;
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid host bits
        let cidr = 24;
        let address = WgIpAddr::from("0.0.0.1".parse::<Ipv4Addr>().unwrap());
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());

        // Valid host bits
        let cidr = 24;
        let address = WgIpAddr::from("255.255.255.0".parse::<Ipv4Addr>().unwrap());
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // 0.0.0.0/0
        let cidr = 0;
        let address = WgIpAddr::from("0.0.0.0".parse::<Ipv4Addr>().unwrap());
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid CIDR
        let cidr = 33;
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());
    }

    #[test]
    fn test_wg_allowed_ip_v6() {
        // Valid: /128 prefix
        let address_family = AF_INET6;
        let address = WgIpAddr::from("::1".parse::<Ipv6Addr>().unwrap());
        let cidr = 128;
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid host bits
        let cidr = 127;
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());

        // Valid host bits
        let address = WgIpAddr::from(
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe"
                .parse::<Ipv6Addr>()
                .unwrap(),
        );
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // ::/0
        let cidr = 0;
        let address = WgIpAddr::from("::".parse::<Ipv6Addr>().unwrap());
        WgAllowedIp::new(address, address_family, cidr).unwrap();

        // Invalid CIDR
        let cidr = 129;
        assert!(WgAllowedIp::new(address, address_family, cidr).is_err());
    }

    unsafe extern "system" fn null_fn() -> isize {
        unreachable!("unexpected call of function")
    }
}
