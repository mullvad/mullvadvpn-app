mod device;
mod service;
mod split_tunnel;

use std::{io, path::PathBuf, ptr};
use windows_sys::Win32::{
    Devices::DeviceAndDriverInstallation::GUID_DEVCLASS_NET,
    Foundation::{FALSE, FreeLibrary, HMODULE},
    System::LibraryLoader::{GetProcAddress, LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW},
};

use crate::Error;
use device::DeviceInfoSet;

// Wintun adapter GUID that may have been left behind
const WINTUN_ABANDONED_GUID: &str = "{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}";

/// Reset split tunnel driver state, stop and delete the `mullvad-split-tunnel`
/// service.
pub fn remove_split_tunnel() -> Result<(), Error> {
    if service::service_is_running(split_tunnel::SERVICE_NAME).map_err(Error::ServiceControl)? {
        split_tunnel::reset_driver_state()?;
    }

    service::stop_and_delete_service(split_tunnel::SERVICE_NAME).map_err(Error::ServiceControl)?;

    Ok(())
}

/// Dynamically load `wintun.dll` (from the same directory as this executable)
/// and call `WintunDeleteDriver`.
pub fn remove_wintun() -> Result<(), Error> {
    // SAFETY: `WintunDeleteDriver` exported by `wintun.dll` has the
    // signature `BOOL WintunDeleteDriver(void)`.
    unsafe { call_delete_driver_fn("wintun.dll", "WintunDeleteDriver") }?;
    tracing::info!("Removed Wintun driver");
    Ok(())
}

/// Find and uninstall an abandoned Wintun network adapter with the well-known
/// interface GUID `{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}`.
pub fn remove_wintun_abandoned_device() -> Result<(), Error> {
    let device_info_set =
        DeviceInfoSet::new(GUID_DEVCLASS_NET).map_err(Error::DeviceEnumeration)?;
    for info in device_info_set.iter() {
        let device_info = info.map_err(Error::DeviceEnumeration)?;
        if let Ok(id) = device_info.get_device_net_cfg_instance_id()
            && id.eq_ignore_ascii_case(WINTUN_ABANDONED_GUID)
        {
            device_info
                .uninstall_device()
                .map_err(Error::DeviceEnumeration)?;
            return Ok(());
        }
    }
    Ok(())
}

/// Dynamically load `mullvad-wireguard.dll` (from the same directory as this
/// executable) and call `WireGuardDeleteDriver`.
pub fn remove_wg_nt() -> Result<(), Error> {
    // SAFETY: `WireGuardDeleteDriver` exported by `mullvad-wireguard.dll` has
    // the signature `BOOL WireGuardDeleteDriver(void)`.
    unsafe { call_delete_driver_fn("mullvad-wireguard.dll", "WireGuardDeleteDriver") }?;
    tracing::info!("Removed WireGuardNT driver");
    Ok(())
}

struct Dll {
    handle: HMODULE,
}

impl Dll {
    /// - path: Path to a dll library.
    fn load(path: PathBuf) -> Result<Self, Error> {
        let dll_path = encode_as_utf16(path);
        // SAFETY: `dll_path` is a NUL-terminated UTF-16 string; the reserved handle is NULL as required.
        let handle: HMODULE = unsafe {
            LoadLibraryExW(
                dll_path.as_ptr(),
                ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            )
        };
        if handle.is_null() {
            return Err(Error::LoadLibrary(io::Error::last_os_error()));
        }
        Ok(Self { handle })
    }

    /// Return a function pointer for `fn_name`.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress>
    fn get_fn_address(&self, fn_name: &str) -> Result<GetProcAddressFn, Error> {
        let fn_name_bytes: Vec<u8> = fn_name.bytes().chain(std::iter::once(0u8)).collect();
        // SAFETY: `handle` is a valid module handle; `fn_name_bytes` is NUL-terminated.
        let proc = unsafe { GetProcAddress(self.handle, fn_name_bytes.as_ptr()) };
        proc.ok_or_else(|| Error::LoadLibrary(io::Error::last_os_error()))
    }
}

impl Drop for Dll {
    fn drop(&mut self) {
        // SAFETY: `handle` is a valid module handle that we own and was created by calling
        // LoadLibraryEx.
        unsafe { FreeLibrary(self.handle) };
    }
}

type GetProcAddressFn = unsafe extern "system" fn() -> isize;
type DeleteDriverFn = unsafe extern "system" fn() -> windows_sys::core::BOOL;

/// Load `dll_name` from the directory of the current executable, look up
/// `fn_name`, call it, and return an error if it returns FALSE.
///
/// # Safety
///
/// The exported function named `fn_name` in `dll_name` must have the signature
/// `extern "system" fn() -> BOOL` and be safe to call with no arguments.
unsafe fn call_delete_driver_fn(dll_name: &str, fn_name: &str) -> Result<(), Error> {
    let dll_path = dll_path(dll_name).map_err(Error::LoadLibrary)?;
    let dll = Dll::load(dll_path)?;
    let proc = dll.get_fn_address(fn_name)?;
    // SAFETY: `fn_name` in the dll `dll_name` has system ABI signature `fn() -> BOOL`.
    let delete_driver: DeleteDriverFn = unsafe { std::mem::transmute(proc) };
    // SAFETY: The function takes no arguments and is safe to call from any thread.
    if unsafe { delete_driver() } == FALSE {
        return Err(Error::DeleteDriver);
    }
    Ok(())
}

/// Returns the full filesystem path for the sought dll as a NUL-terminated UTF-16 string.
///
/// The dll must be part of the Mullvad VPN app installation directory, or whatever the current
/// directory of this eexcutable is.
fn dll_path(dll_name: &str) -> io::Result<PathBuf> {
    std::env::current_exe().map(|path| path.parent().unwrap().join(dll_name))
}

fn encode_as_utf16(path: PathBuf) -> Vec<u16> {
    path.to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}
