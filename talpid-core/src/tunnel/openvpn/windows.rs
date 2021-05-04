use std::{
    ffi::CStr,
    fmt, io, iter, mem,
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
        minwindef::{BOOL, FARPROC, HINSTANCE, HMODULE},
        netioapi::{
            CancelMibChangeNotify2, ConvertInterfaceLuidToGuid, GetIpInterfaceEntry,
            NotifyIpInterfaceChange, MIB_IPINTERFACE_ROW,
        },
        ntdef::FALSE,
        winerror::NO_ERROR,
    },
    um::{
        libloaderapi::{
            FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH,
        },
        winreg::REGSAM,
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};


/// Longest possible adapter name (in characters), including null terminator
const MAX_ADAPTER_NAME: usize = 128;

type WintunOpenAdapterFn =
    unsafe extern "stdcall" fn(pool: *const u16, name: *const u16) -> RawHandle;

type WintunCreateAdapterFn = unsafe extern "stdcall" fn(
    pool: *const u16,
    name: *const u16,
    requested_guid: *const GUID,
    reboot_required: *mut BOOL,
) -> RawHandle;

type WintunFreeAdapterFn = unsafe extern "stdcall" fn(adapter: RawHandle);

type WintunDeleteAdapterFn = unsafe extern "stdcall" fn(
    adapter: RawHandle,
    force_close_sessions: BOOL,
    reboot_required: *mut BOOL,
) -> BOOL;

type WintunGetAdapterNameFn =
    unsafe extern "stdcall" fn(adapter: RawHandle, name: *mut u16) -> BOOL;

type WintunGetAdapterLuidFn = unsafe extern "stdcall" fn(adapter: RawHandle, luid: *mut NET_LUID);

type WintunLoggerCbFn = extern "stdcall" fn(WintunLoggerLevel, *const u16);

type WintunSetLoggerFn = unsafe extern "stdcall" fn(Option<WintunLoggerCbFn>);

#[repr(C)]
#[allow(dead_code)]
enum WintunLoggerLevel {
    Info,
    Warn,
    Err,
}


pub struct WintunDll {
    handle: HINSTANCE,
    func_open: WintunOpenAdapterFn,
    func_create: WintunCreateAdapterFn,
    func_free: WintunFreeAdapterFn,
    func_delete: WintunDeleteAdapterFn,
    func_get_adapter_name: WintunGetAdapterNameFn,
    func_get_adapter_luid: WintunGetAdapterLuidFn,
    func_set_logger: WintunSetLoggerFn,
}

unsafe impl Send for WintunDll {}
unsafe impl Sync for WintunDll {}

type RebootRequired = bool;

/// A new Wintun adapter that is destroyed when dropped.
#[derive(Debug)]
pub struct TemporaryWintunAdapter {
    pub adapter: WintunAdapter,
}

impl TemporaryWintunAdapter {
    pub fn create(
        dll_handle: Arc<WintunDll>,
        pool: &U16CStr,
        name: &U16CStr,
        requested_guid: Option<GUID>,
    ) -> io::Result<(Self, RebootRequired)> {
        let (adapter, reboot_required) =
            WintunAdapter::create(dll_handle, pool, name, requested_guid)?;
        Ok((TemporaryWintunAdapter { adapter }, reboot_required))
    }

    pub fn adapter(&self) -> &WintunAdapter {
        &self.adapter
    }
}

impl Drop for TemporaryWintunAdapter {
    fn drop(&mut self) {
        if let Err(error) = unsafe {
            self.adapter
                .dll_handle
                .delete_adapter(self.adapter.handle, true)
        } {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to delete Wintun adapter")
            );
        }
    }
}

/// Represents a Wintun adapter.
pub struct WintunAdapter {
    dll_handle: Arc<WintunDll>,
    handle: RawHandle,
}

impl fmt::Debug for WintunAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WintunAdapter")
            .field("handle", &self.handle)
            .finish()
    }
}

unsafe impl Send for WintunAdapter {}

impl WintunAdapter {
    pub fn open(dll_handle: Arc<WintunDll>, pool: &U16CStr, name: &U16CStr) -> io::Result<Self> {
        Ok(Self {
            handle: dll_handle.open_adapter(pool, name)?,
            dll_handle,
        })
    }

    pub fn create(
        dll_handle: Arc<WintunDll>,
        pool: &U16CStr,
        name: &U16CStr,
        requested_guid: Option<GUID>,
    ) -> io::Result<(Self, RebootRequired)> {
        let (handle, restart_required) = dll_handle.create_adapter(pool, name, requested_guid)?;
        Ok((Self { dll_handle, handle }, restart_required))
    }

    pub fn delete(self, force_close_sessions: bool) -> io::Result<RebootRequired> {
        unsafe {
            self.dll_handle
                .delete_adapter(self.handle, force_close_sessions)
        }
    }

    pub fn name(&self) -> io::Result<U16CString> {
        unsafe { self.dll_handle.get_adapter_name(self.handle) }
    }

    pub fn luid(&self) -> NET_LUID {
        unsafe { self.dll_handle.get_adapter_luid(self.handle) }
    }

    pub fn guid(&self) -> io::Result<GUID> {
        let mut guid = mem::MaybeUninit::zeroed();
        let result = unsafe { ConvertInterfaceLuidToGuid(&self.luid(), guid.as_mut_ptr()) };
        if result != NO_ERROR {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { guid.assume_init() })
    }
}

impl Drop for WintunAdapter {
    fn drop(&mut self) {
        unsafe { self.dll_handle.free_adapter(self.handle) };
    }
}

impl WintunDll {
    pub fn new(resource_dir: &Path) -> io::Result<Self> {
        let wintun_dll: Vec<u16> = resource_dir
            .join("wintun.dll")
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0u16))
            .collect();

        let handle = unsafe {
            LoadLibraryExW(
                wintun_dll.as_ptr(),
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
        Ok(WintunDll {
            handle,
            func_open: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunOpenAdapter\0").unwrap(),
                )?)
            },
            func_create: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunCreateAdapter\0").unwrap(),
                )?)
            },
            func_delete: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunDeleteAdapter\0").unwrap(),
                )?)
            },
            func_free: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunFreeAdapter\0").unwrap(),
                )?)
            },
            func_get_adapter_name: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunGetAdapterName\0").unwrap(),
                )?)
            },
            func_get_adapter_luid: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunGetAdapterLUID\0").unwrap(),
                )?)
            },
            func_set_logger: unsafe {
                std::mem::transmute(get_proc_fn(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunSetLogger\0").unwrap(),
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

    pub unsafe fn delete_adapter(
        &self,
        adapter: RawHandle,
        force_close_sessions: bool,
    ) -> io::Result<RebootRequired> {
        let mut reboot_required = 0;
        let force_close_sessions = if force_close_sessions { 1 } else { 0 };
        let result = (self.func_delete)(adapter, force_close_sessions, &mut reboot_required);
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

    pub fn activate_logging(self: &Arc<Self>) -> WintunLoggerHandle {
        WintunLoggerHandle::from_handle(self.clone())
    }

    fn set_logger(&self, logger: Option<WintunLoggerCbFn>) {
        unsafe { (self.func_set_logger)(logger) };
    }
}

impl Drop for WintunDll {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.handle) };
    }
}

pub struct WintunLoggerHandle {
    dll_handle: Arc<WintunDll>,
}

impl WintunLoggerHandle {
    fn from_handle(dll_handle: Arc<WintunDll>) -> Self {
        dll_handle.set_logger(Some(Self::callback));
        Self { dll_handle }
    }

    extern "stdcall" fn callback(level: WintunLoggerLevel, message: *const u16) {
        if message.is_null() {
            return;
        }
        let message = unsafe { U16CStr::from_ptr_str(message) };

        use WintunLoggerLevel::*;

        match level {
            Info => log::info!("[Wintun] {}", message.to_string_lossy()),
            Warn => log::warn!("[Wintun] {}", message.to_string_lossy()),
            Err => log::error!("[Wintun] {}", message.to_string_lossy()),
        }
    }
}

impl fmt::Debug for WintunLoggerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WintunLogger").finish()
    }
}

impl Drop for WintunLoggerHandle {
    fn drop(&mut self) {
        self.dll_handle.set_logger(None);
    }
}

/// Obtain a string representation for a GUID object.
pub fn string_from_guid(guid: &GUID) -> String {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    use winapi::um::combaseapi::StringFromGUID2;

    let mut buffer = [0u16; 40];
    let length = unsafe { StringFromGUID2(guid, &mut buffer[0] as *mut _, buffer.len() as i32 - 1) }
        as usize;
    if length > 0 {
        let length = length - 1;
        OsString::from_wide(&buffer[0..length])
            .to_string_lossy()
            .to_string()
    } else {
        "".to_string()
    }
}

pub fn find_adapter_registry_key(find_guid: &str, permissions: REGSAM) -> io::Result<RegKey> {
    let net_devs = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}",
        permissions,
    )?;
    let find_guid = find_guid.to_lowercase();

    for subkey_name in net_devs.enum_keys() {
        let subkey_name = match subkey_name {
            Ok(subkey_name) => subkey_name,
            Err(_error) => continue,
        };

        let subkey: io::Result<RegKey> = net_devs.open_subkey_with_flags(&subkey_name, permissions);
        if let Ok(subkey) = subkey {
            let guid_str: io::Result<String> = subkey.get_value("NetCfgInstanceId");
            if let Ok(guid_str) = guid_str {
                if guid_str.to_lowercase() == find_guid {
                    return Ok(subkey);
                }
            }
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "device not found"))
}

pub struct IpNotifierHandle<'a> {
    mutex: Mutex<()>,
    callback: Option<Box<dyn FnMut(&MIB_IPINTERFACE_ROW, u32) + Send + 'a>>,
    handle: RawHandle,
}

impl<'a> Drop for IpNotifierHandle<'a> {
    fn drop(&mut self) {
        // Inner callback may be called while destructing
        unsafe { CancelMibChangeNotify2(self.handle as *mut _) };

        let _ = self
            .mutex
            .lock()
            .expect("NotifyIpInterfaceChange mutex poisoned");
        let _ = self.callback.take();
    }
}

unsafe extern "system" fn inner_callback(
    context: *mut winapi::ctypes::c_void,
    row: *mut MIB_IPINTERFACE_ROW,
    notify_type: u32,
) {
    let context = &mut *(context as *mut IpNotifierHandle<'_>);
    let _ = context
        .mutex
        .lock()
        .expect("NotifyIpInterfaceChange mutex poisoned");

    if let Some(ref mut callback) = context.callback {
        callback(&*row, notify_type);
    }
}

pub fn notify_ip_interface_change<'a, T: FnMut(&MIB_IPINTERFACE_ROW, u32) + Send + 'a>(
    callback: T,
    family: u16,
) -> io::Result<Box<IpNotifierHandle<'a>>> {
    let mut context = Box::new(IpNotifierHandle {
        mutex: Mutex::default(),
        callback: Some(Box::new(callback)),
        handle: std::ptr::null_mut(),
    });

    let status = unsafe {
        NotifyIpInterfaceChange(
            family,
            Some(inner_callback),
            &mut *context as *mut _ as *mut _,
            FALSE,
            (&mut context.handle) as *mut _,
        )
    };

    if status != NO_ERROR {
        return Err(io::Error::last_os_error());
    }

    Ok(context)
}

pub fn get_ip_interface_entry(family: u16, luid: &NET_LUID) -> io::Result<MIB_IPINTERFACE_ROW> {
    let mut row: MIB_IPINTERFACE_ROW = unsafe { mem::zeroed() };
    row.Family = family;
    row.InterfaceLuid = *luid;

    let result = unsafe { GetIpInterfaceEntry(&mut row as *mut _) };
    if result != NO_ERROR {
        return Err(io::Error::last_os_error());
    }

    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_proc_fn(_handle: HMODULE, _symbol: &CStr) -> io::Result<FARPROC> {
        Ok(std::ptr::null_mut())
    }

    #[test]
    fn test_wintun_imports() {
        WintunDll::new_inner(ptr::null_mut(), get_proc_fn).unwrap();
    }

    #[test]
    fn guid_to_string() {
        let guids = [
            (
                "{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}",
                GUID {
                    Data1: 0xAFE43773,
                    Data2: 0xE1F8,
                    Data3: 0x4EBB,
                    Data4: [0x85, 0x36, 0x57, 0x6A, 0xB8, 0x6A, 0xFE, 0x9A],
                },
            ),
            (
                "{00000000-0000-0000-0000-000000000000}",
                GUID {
                    Data1: 0,
                    Data2: 0,
                    Data3: 0,
                    Data4: [0; 8],
                },
            ),
        ];

        for (expected_str, guid) in &guids {
            assert_eq!(
                string_from_guid(guid).as_str().to_lowercase(),
                expected_str.to_lowercase()
            );
        }
    }
}
