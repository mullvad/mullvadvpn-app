use std::{
    ffi::CStr,
    fmt, io, iter,
    os::windows::{ffi::OsStrExt, io::RawHandle},
    path::Path,
    ptr,
    sync::Arc,
};
use talpid_types::ErrorExt;
use widestring::{U16CStr, U16CString};
use winapi::{
    shared::{
        guiddef::GUID,
        minwindef::{BOOL, FARPROC, HINSTANCE, HMODULE},
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


pub struct WintunDll {
    handle: HINSTANCE,
    func_open: WintunOpenAdapterFn,
    func_create: WintunCreateAdapterFn,
    func_free: WintunFreeAdapterFn,
    func_delete: WintunDeleteAdapterFn,
    func_get_adapter_name: WintunGetAdapterNameFn,
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

        Ok(WintunDll {
            handle,
            func_open: unsafe {
                std::mem::transmute(Self::get_proc_address(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunOpenAdapter\0").unwrap(),
                )?)
            },
            func_create: unsafe {
                std::mem::transmute(Self::get_proc_address(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunCreateAdapter\0").unwrap(),
                )?)
            },
            func_delete: unsafe {
                std::mem::transmute(Self::get_proc_address(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunDeleteAdapter\0").unwrap(),
                )?)
            },
            func_free: unsafe {
                std::mem::transmute(Self::get_proc_address(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunFreeAdapter\0").unwrap(),
                )?)
            },
            func_get_adapter_name: unsafe {
                std::mem::transmute(Self::get_proc_address(
                    handle,
                    CStr::from_bytes_with_nul(b"WintunGetAdapterName\0").unwrap(),
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
}

impl Drop for WintunDll {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.handle) };
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
