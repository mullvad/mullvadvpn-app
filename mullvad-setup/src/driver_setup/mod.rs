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

// Hardware ID used by the legacy `mullvad-wireguard.dll` fork of WireGuardNT.
const MULLVAD_WG_HARDWARE_ID: &str = "MullvadWireGuard";

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

/// Dynamically load `wireguard.dll` (from the same directory as this
/// executable) and call `WireGuardDeleteDriver`.
pub fn remove_wg_nt() -> Result<(), Error> {
    // SAFETY: `WireGuardDeleteDriver` exported by `wireguard.dll` has
    // the signature `BOOL WireGuardDeleteDriver(void)`.
    unsafe { call_delete_driver_fn("wireguard.dll", "WireGuardDeleteDriver") }?;
    tracing::info!("Removed WireGuardNT driver");
    Ok(())
}

/// Remove leftovers from the legacy `mullvad-wireguard.dll` fork: network
/// adapters with hardware ID `MullvadWireGuard` and the OEM INF in the driver
/// store. Unlike [`remove_wg_nt`] this needs to not depend on `mullvad-wireguard.dll`
/// is no longer shipped.
pub fn remove_wg_nt_abandoned() -> Result<(), Error> {
    let removed_devices = uninstall_devices_by_hardware_id(MULLVAD_WG_HARDWARE_ID)?;
    if removed_devices > 0 {
        tracing::info!("Uninstalled {removed_devices} legacy MullvadWireGuard device(s)");
    }

    remove_network_driver_by_hardware_id(MULLVAD_WG_HARDWARE_ID)?;

    Ok(())
}

/// Enumerate `GUID_DEVCLASS_NET` and uninstall every device whose hardware ID
/// list contains `hardware_id` (case-insensitive). Returns the number of
/// devices that were uninstalled.
//
// Uninstalling a device invalidates the device info set, so we rebuild it
// after each removal.
// TODO: Check if this is actually true.
fn uninstall_devices_by_hardware_id(hardware_id: &str) -> Result<usize, Error> {
    let mut removed = 0;
    loop {
        let device_info_set =
            DeviceInfoSet::new(GUID_DEVCLASS_NET).map_err(Error::DeviceEnumeration)?;
        let mut found = None;
        for info in device_info_set.iter() {
            let device_info = info.map_err(Error::DeviceEnumeration)?;
            if let Ok(ids) = device_info.get_hardware_ids()
                && ids.iter().any(|id| id.eq_ignore_ascii_case(hardware_id))
            {
                found = Some(device_info);
                break;
            }
        }
        match found {
            Some(device_info) => {
                device_info
                    .uninstall_device()
                    .map_err(Error::DeviceEnumeration)?;
                removed += 1;
            }
            None => return Ok(removed),
        }
    }
}

/// Uninstall every INF in the driver store that matches the class `GUID_DEVCLASS_NET`
/// and the given hardware ID.
fn remove_network_driver_by_hardware_id(hardware_id: &str) -> Result<(), Error> {
    let inf_names = device::list_compatible_driver_inf_names(GUID_DEVCLASS_NET, hardware_id)
        .map_err(Error::DeviceEnumeration)?;
    for inf_name in inf_names {
        device::uninstall_oem_inf(&inf_name).map_err(Error::DeviceEnumeration)?;
        tracing::info!(
            "Uninstalled legacy MullvadWireGuard OEM INF: {}",
            inf_name.to_string_lossy()
        );
    }
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
