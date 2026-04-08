mod device;
mod service;
mod split_tunnel;

use std::{io, ptr};
use windows_sys::Win32::{
    Devices::DeviceAndDriverInstallation::GUID_DEVCLASS_NET,
    Foundation::{FALSE, FreeLibrary, HMODULE},
    System::LibraryLoader::{GetProcAddress, LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW},
};

const SPLIT_TUNNEL_SERVICE_NAME: &str = "mullvad-split-tunnel";

// Wintun adapter GUID that may have been left behind
const WINTUN_ABANDONED_GUID: &str = "{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}";

/// Reset split tunnel driver state, stop and delete the `mullvad-split-tunnel`
/// service.
pub fn remove_split_tunnel() -> Result<(), crate::Error> {
    if service::service_is_running(SPLIT_TUNNEL_SERVICE_NAME)
        .map_err(crate::Error::ServiceControl)?
    {
        split_tunnel::reset_driver_state()?;
    }

    service::stop_and_delete_service(SPLIT_TUNNEL_SERVICE_NAME)
        .map_err(crate::Error::ServiceControl)?;

    Ok(())
}

/// Dynamically load `wintun.dll` (from the same directory as this executable)
/// and call `WintunDeleteDriver`.
pub fn remove_wintun() -> Result<(), crate::Error> {
    // SAFETY: `WintunDeleteDriver` exported by `wintun.dll` has the
    // signature `BOOL WintunDeleteDriver(void)`.
    unsafe { call_delete_driver_fn("wintun.dll", "WintunDeleteDriver") }?;
    tracing::info!("Removed Wintun driver");
    Ok(())
}

/// Find and uninstall an abandoned Wintun network adapter with the well-known
/// interface GUID `{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}`.
pub fn remove_wintun_abandoned_device() -> Result<(), crate::Error> {
    device::find_and_uninstall_device(GUID_DEVCLASS_NET, |set| {
        // SAFETY: `set` is passed in by `find_and_uninstall_device`
        // after being obtained from `SetupDiGetClassDevsW` / `SetupDiEnumDeviceInfo`.
        match unsafe { device::get_device_net_cfg_instance_id(set) } {
            Ok(id) => id.eq_ignore_ascii_case(WINTUN_ABANDONED_GUID),
            Err(_) => false,
        }
    })
    .map_err(crate::Error::DeviceEnumeration)?;

    Ok(())
}

/// Dynamically load `mullvad-wireguard.dll` (from the same directory as this
/// executable) and call `WireGuardDeleteDriver`.
pub fn remove_wg_nt() -> Result<(), crate::Error> {
    // SAFETY: `WireGuardDeleteDriver` exported by `mullvad-wireguard.dll` has
    // the signature `BOOL WireGuardDeleteDriver(void)`.
    unsafe { call_delete_driver_fn("mullvad-wireguard.dll", "WireGuardDeleteDriver") }?;
    tracing::info!("Removed WireGuardNT driver");
    Ok(())
}

/// Load `dll_name` from the directory of the current executable, look up
/// `fn_name`, call it, and return an error if it returns FALSE.
///
/// # Safety
///
/// The exported function named `fn_name` in `dll_name` must have the signature
/// `extern "system" fn() -> BOOL` and be safe to call with no arguments.
unsafe fn call_delete_driver_fn(dll_name: &str, fn_name: &str) -> Result<(), crate::Error> {
    let exe_path = std::env::current_exe().map_err(crate::Error::LoadLibrary)?;
    let dll_path = exe_path
        .parent()
        .expect("current_exe always has a parent directory")
        .join(dll_name);

    let dll_path_wide: Vec<u16> = dll_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: `dll_path_wide` is a NUL-terminated UTF-16 string; the reserved handle is NULL as required.
    let handle: HMODULE = unsafe {
        LoadLibraryExW(
            dll_path_wide.as_ptr(),
            ptr::null_mut(),
            LOAD_WITH_ALTERED_SEARCH_PATH,
        )
    };

    if handle.is_null() {
        return Err(crate::Error::LoadLibrary(io::Error::last_os_error()));
    }

    let fn_name_bytes: Vec<u8> = fn_name.bytes().chain(std::iter::once(0u8)).collect();

    // SAFETY: `handle` is a valid module handle; `fn_name_bytes` is NUL-terminated.
    let proc = unsafe { GetProcAddress(handle, fn_name_bytes.as_ptr()) };

    let Some(proc) = proc else {
        // SAFETY: `handle` is a valid module handle that we own.
        unsafe { FreeLibrary(handle) };
        return Err(crate::Error::LoadLibrary(io::Error::last_os_error()));
    };

    // The function has the signature: `fn() -> BOOL`
    type DeleteDriverFn = unsafe extern "system" fn() -> windows_sys::core::BOOL;
    // SAFETY: `WintunDeleteDriver`/`WireGuardDeleteDriver` both match the
    // `fn() -> BOOL` system ABI signature.
    let delete_driver: DeleteDriverFn = unsafe { std::mem::transmute(proc) };

    // SAFETY: The function takes no arguments and is safe to call from any thread.
    let result = unsafe { delete_driver() };

    // SAFETY: `handle` is a valid module handle that we own and no longer use.
    unsafe { FreeLibrary(handle) };

    if result == FALSE {
        return Err(crate::Error::DeleteDriver);
    }

    Ok(())
}
