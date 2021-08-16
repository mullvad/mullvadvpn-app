use super::{config::Config, stats::Stats, Tunnel};
use bitflags::bitflags;
use ipnetwork::IpNetwork;
use lazy_static::lazy_static;
use std::{
    ffi::CStr,
    fmt, io, iter, mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
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
    Data1: 0xAFE43773,
    Data2: 0xE1F8,
    Data3: 0x4EBB,
    Data4: [0x85, 0x36, 0x57, 0x6A, 0xB8, 0x6A, 0xFE, 0x9A],
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

type RebootRequired = bool;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to load WireGuardNT
    #[error(display = "Failed to load wireguard.dll")]
    DllError(#[error(source)] io::Error),

    /// Failed to create tunnel interface
    #[error(display = "Failed to create WireGuard device")]
    CreateTunnelDeviceError(#[error(source)] io::Error),

    /// Failed to delete tunnel interface
    #[error(display = "Failed to delete WireGuard device")]
    DeleteTunnelDeviceError(#[error(source)] io::Error),

    /// Failed to obtain tunnel interface alias
    #[error(display = "Failed to obtain interface name")]
    ObtainAliasError(#[error(source)] io::Error),

    /// Failed to set WireGuard tunnel config on device
    #[error(display = "Failed to set tunnel WireGuard config")]
    SetWireGuardConfigError(#[error(source)] io::Error),

    /// Failed to set the tunnel state to up
    #[error(display = "Failed to enable the tunnel adapter")]
    EnableTunnelError(#[error(source)] io::Error),
}

pub struct WgNtTunnel {
    device: Option<WgNtAdapter>,
    interface_luid: NET_LUID,
    interface_name: String,
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
#[derive(Clone, Copy)]
#[repr(C, align(8))]
struct WgPeer {
    flags: WgPeerFlag,
    reserved: u32,
    public_key: [u8; WIREGUARD_KEY_LENGTH],
    preshared_key: [u8; WIREGUARD_KEY_LENGTH],
    persistent_keepalive: u16,
    endpoint: SOCKADDR_INET,
    tx_bytes: u64,
    rx_bytes: u64,
    last_handshake: u64,
    allowed_ips_count: u32,
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
#[derive(Clone, Copy)]
#[repr(C, align(8))]
struct WgInterface {
    flags: WgInterfaceFlag,
    listen_port: u16,
    private_key: [u8; WIREGUARD_KEY_LENGTH],
    public_key: [u8; WIREGUARD_KEY_LENGTH],
    peers_count: u32,
}

/// See `WIREGUARD_ADAPTER_LOG_STATE` at https://git.zx2c4.com/wireguard-nt/tree/api/wireguard.h.
#[derive(Clone, Copy)]
#[repr(C)]
enum WgAdapterState {
    Down,
    Up,
}


impl WgNtTunnel {
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        resource_dir: &Path,
    ) -> Result<Self> {
        let dll = load_wg_nt_dll(resource_dir)?;

        let (device, reboot_required) = WgNtAdapter::create(
            dll,
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
        device
            .set_config(config)
            .map_err(Error::SetWireGuardConfigError)?;
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

    fn set_config(&self, config: &Config) -> io::Result<()> {
        let config_buffer = serialize_config(config);
        unsafe {
            self.dll_handle
                .set_config(self.handle, config_buffer.as_ptr(), config_buffer.len())
        }
    }

    fn get_config(&self) -> io::Result<(WgInterface, Vec<(WgPeer, Vec<WgAllowedIp>)>)> {
        Ok(unsafe { deserialize_config(&self.dll_handle.get_config(self.handle)?) })
    }

    fn set_state(&self, state: WgAdapterState) -> io::Result<()> {
        unsafe { self.dll_handle.set_adapter_state(self.handle, state) }
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
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardOpenAdapter\0").unwrap(),
                )?)
            },
            func_create: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardCreateAdapter\0").unwrap(),
                )?)
            },
            func_delete: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardDeleteAdapter\0").unwrap(),
                )?)
            },
            func_free: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardFreeAdapter\0").unwrap(),
                )?)
            },
            func_get_adapter_luid: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetAdapterLUID\0").unwrap(),
                )?)
            },
            func_get_adapter_name: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetAdapterName\0").unwrap(),
                )?)
            },
            func_set_configuration: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardSetConfiguration\0").unwrap(),
                )?)
            },
            func_get_configuration: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardGetConfiguration\0").unwrap(),
                )?)
            },
            func_set_adapter_state: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WireGuardSetAdapterState\0").unwrap(),
                )?)
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

fn serialize_config(config: &Config) -> Vec<u8> {
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
            endpoint: convert_socket_address(peer.endpoint),
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

            let wg_allowed_ip = WgAllowedIp {
                address,
                address_family,
                cidr: allowed_ip.prefix() as u8,
            };

            buffer.extend_from_slice(unsafe { as_u8_slice(&wg_allowed_ip) });
        }
    }

    buffer
}

unsafe fn deserialize_config(config: &[u8]) -> (WgInterface, Vec<(WgPeer, Vec<WgAllowedIp>)>) {
    let (head, mut tail) = config.split_at(mem::size_of::<WgInterface>());
    let interface: WgInterface = *(head.as_ptr() as *const WgInterface);

    let mut peers = vec![];
    for _ in 0..interface.peers_count {
        let (peer_data, new_tail) = tail.split_at(mem::size_of::<WgPeer>());
        let peer: WgPeer = *(peer_data.as_ptr() as *const WgPeer);
        tail = new_tail;

        let mut allowed_ips = vec![];

        for _ in 0..peer.allowed_ips_count {
            let (allowed_ip_data, new_tail) = tail.split_at(mem::size_of::<WgAllowedIp>());
            let allowed_ip: WgAllowedIp = *(allowed_ip_data.as_ptr() as *const WgAllowedIp);
            tail = new_tail;
            allowed_ips.push(allowed_ip);
        }

        peers.push((peer, allowed_ips));
    }

    (interface, peers)
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
        }
    }

    sockaddr
}

impl Tunnel for WgNtTunnel {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn get_interface_luid(&self) -> u64 {
        self.interface_luid.Value
    }

    fn get_tunnel_stats(&self) -> std::result::Result<Stats, super::TunnelError> {
        if let Some(ref device) = self.device {
            let (_interface, peers) = device.get_config().map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to obtain NT tunnel config")
                );

                // TODO: Improve error
                super::TunnelError::StatsError(super::stats::Error::KeyNotFoundError)
            })?;

            let mut tx_bytes = 0;
            let mut rx_bytes = 0;

            for (peer, _allowed_ips) in &peers {
                tx_bytes += peer.tx_bytes;
                rx_bytes += peer.rx_bytes;
            }

            Ok(Stats { tx_bytes, rx_bytes })
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
