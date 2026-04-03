use std::{io, mem, ptr};
use windows_sys::{
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                DICS_FLAG_GLOBAL, DIGCF_PRESENT, DIREG_DRV, DiUninstallDevice, HDEVINFO,
                SP_DEVINFO_DATA, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
                SetupDiGetClassDevsW, SetupDiGetDevicePropertyW, SetupDiOpenDevRegKey,
            },
            Properties::{DEVPKEY_NAME, DEVPROPTYPE},
        },
        Foundation::{
            DEVPROPKEY, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, ERROR_NOT_FOUND, FALSE,
            INVALID_HANDLE_VALUE,
        },
        System::Registry::{HKEY, KEY_READ, RRF_RT_REG_SZ, RegCloseKey, RegGetValueW},
    },
    core::GUID,
};

struct DeviceInfoSet(HDEVINFO);

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        // SAFETY: `self.0` was returned by `SetupDiGetClassDevsW` and is only destroyed here.
        unsafe {
            SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

/// Enumerate devices of the given class. If `filter` returns true for a device,
/// uninstall it and return `Ok(true)`.  Returns `Ok(false)` if no matching device
/// is found.
pub fn find_and_uninstall_device(
    class_guid: GUID,
    filter: impl Fn(HDEVINFO, &SP_DEVINFO_DATA) -> bool,
) -> Result<bool, io::Error> {
    // SAFETY: `class_guid` points to a valid GUID; the other pointer args are documented as optional.
    let device_info_set = unsafe {
        SetupDiGetClassDevsW(
            &raw const class_guid,
            ptr::null(),
            ptr::null_mut(),
            DIGCF_PRESENT,
        )
    };

    if device_info_set == -1 {
        return Err(io::Error::last_os_error());
    }

    let _set_guard = DeviceInfoSet(device_info_set);

    let mut index: u32 = 0;
    loop {
        // SAFETY: `SP_DEVINFO_DATA` is a POD struct; zero is a valid bit pattern.
        let mut device_info: SP_DEVINFO_DATA = unsafe { mem::zeroed() };
        device_info.cbSize = mem::size_of::<SP_DEVINFO_DATA>() as u32;

        // SAFETY: `device_info_set` is a valid HDEVINFO; `device_info` has `cbSize` set.
        let result = unsafe { SetupDiEnumDeviceInfo(device_info_set, index, &raw mut device_info) };

        if result == FALSE {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(ERROR_NO_MORE_ITEMS as i32) {
                return Ok(false);
            }
            return Err(err);
        }

        index += 1;

        if filter(device_info_set, &device_info) {
            // SAFETY: `device_info_set` is valid (checked above) and `device_info`
            // was just enumerated from it via `SetupDiEnumDeviceInfo`.
            unsafe { uninstall_device(device_info_set, &device_info) }?;
            return Ok(true);
        }
    }
}

/// Returns `true` if the device's DEVPKEY_NAME matches `name` (case-insensitive).
pub fn device_name_matches(
    device_info_set: HDEVINFO,
    device_info: &SP_DEVINFO_DATA,
    name: &str,
) -> bool {
    match get_device_string_property(device_info_set, device_info, &DEVPKEY_NAME) {
        Ok(Some(prop)) => prop.eq_ignore_ascii_case(name),
        _ => false,
    }
}

/// Read the `NetCfgInstanceId` registry value from a device's driver key.
/// Returns the GUID string (e.g. `{AFE43773-...}`).
///
/// # Safety
///
/// `device_info_set` must be a valid `HDEVINFO` and `device_info` must refer to
/// a device enumerated from that set (e.g. via `SetupDiEnumDeviceInfo`).
pub unsafe fn get_device_net_cfg_instance_id(
    device_info_set: HDEVINFO,
    device_info: &SP_DEVINFO_DATA,
) -> Result<String, io::Error> {
    // SAFETY: Per this function's safety contract, `device_info_set` and `device_info`
    // are valid and belong to the same enumeration.
    let reg_key: HKEY = unsafe {
        SetupDiOpenDevRegKey(
            device_info_set,
            device_info as *const SP_DEVINFO_DATA,
            DICS_FLAG_GLOBAL,
            0,
            DIREG_DRV,
            KEY_READ,
        )
    };

    if std::ptr::eq(reg_key, INVALID_HANDLE_VALUE) {
        return Err(io::Error::last_os_error());
    }

    let value_name: Vec<u16> = "NetCfgInstanceId\0".encode_utf16().collect();
    let mut buffer: Vec<u16> = vec![0u16; 128];
    let mut buffer_byte_len: u32 = (buffer.len() * 2) as u32;

    // SAFETY: `reg_key` is valid; `value_name` is NUL-terminated; `buffer` is sized by `buffer_byte_len`.
    let status = unsafe {
        RegGetValueW(
            reg_key,
            ptr::null(),
            value_name.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buffer.as_mut_ptr() as *mut _,
            &raw mut buffer_byte_len,
        )
    };

    // SAFETY: `reg_key` was opened above and is not used again.
    unsafe { RegCloseKey(reg_key) };

    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }

    let len = buffer
        .iter()
        .position(|&c| c == 0)
        .expect("RegGetValueW guarantees a NUL terminator for REG_SZ");
    Ok(String::from_utf16_lossy(&buffer[..len]))
}

/// Read a string device property. Returns `None` if the property is not set.
fn get_device_string_property(
    device_info_set: HDEVINFO,
    device_info: &SP_DEVINFO_DATA,
    property: &DEVPROPKEY,
) -> Result<Option<String>, io::Error> {
    let mut required_size: u32 = 0;
    let mut prop_type: DEVPROPTYPE = 0;

    // First call: determine buffer size
    let result = unsafe {
        SetupDiGetDevicePropertyW(
            device_info_set,
            device_info as *const SP_DEVINFO_DATA,
            property,
            &raw mut prop_type,
            ptr::null_mut(),
            0,
            &raw mut required_size,
            0,
        )
    };

    if result == FALSE {
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(ERROR_NOT_FOUND as i32) {
            return Ok(None);
        }
        if err.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32) {
            return Err(err);
        }
    }

    if required_size < 2 {
        return Ok(Some(String::new()));
    }

    let mut buffer: Vec<u16> = vec![0u16; (required_size / 2) as usize];

    // Second call: read the value
    let result = unsafe {
        SetupDiGetDevicePropertyW(
            device_info_set,
            device_info as *const SP_DEVINFO_DATA,
            property,
            &raw mut prop_type,
            buffer.as_mut_ptr() as *mut u8,
            required_size,
            ptr::null_mut(),
            0,
        )
    };

    if result == FALSE {
        return Err(io::Error::last_os_error());
    }

    let len = buffer
        .iter()
        .position(|&c| c == 0)
        .expect("SetupDiGetDevicePropertyW guarantees a NUL terminator for DEVPROP_TYPE_STRING");
    Ok(Some(String::from_utf16_lossy(&buffer[..len])))
}

/// # Safety
///
/// `device_info_set` must be a valid `HDEVINFO` and `device_info` must refer to
/// a device enumerated from that set.
unsafe fn uninstall_device(
    device_info_set: HDEVINFO,
    device_info: &SP_DEVINFO_DATA,
) -> Result<(), io::Error> {
    let mut needs_reboot: windows_sys::core::BOOL = 0;
    // SAFETY: Per this function's safety contract, `device_info_set` and `device_info`
    // are valid and belong to the same enumeration. `needs_reboot` is a writable BOOL.
    let result = unsafe {
        DiUninstallDevice(
            ptr::null_mut(),
            device_info_set,
            device_info as *const SP_DEVINFO_DATA,
            0,
            &raw mut needs_reboot,
        )
    };

    if result == FALSE {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
