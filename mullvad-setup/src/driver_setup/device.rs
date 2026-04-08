use std::{io, mem, ptr};
use windows_sys::{
    Win32::{
        Devices::DeviceAndDriverInstallation::{
            DICS_FLAG_GLOBAL, DIGCF_PRESENT, DIREG_DRV, DiUninstallDevice, HDEVINFO,
            SP_DEVINFO_DATA, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
            SetupDiGetClassDevsW, SetupDiOpenDevRegKey,
        },
        Foundation::{ERROR_NO_MORE_ITEMS, FALSE, INVALID_HANDLE_VALUE},
        System::Registry::{HKEY, KEY_READ, RRF_RT_REG_SZ, RegCloseKey, RegGetValueW},
    },
    core::GUID,
    w,
};

pub struct DeviceInfoSet(HDEVINFO);

impl DeviceInfoSet {
    pub fn new(class_guid: GUID) -> io::Result<Self> {
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

        Ok(DeviceInfoSet(device_info_set))
    }

    /// Return an iterator over the device info set.
    pub fn iter(&self) -> DeviceInfoIter<'_> {
        DeviceInfoIter::new(self)
    }

    fn get_device_info(&self, index: u32) -> io::Result<Option<DeviceInfo<'_>>> {
        // SAFETY: `SP_DEVINFO_DATA` is a POD struct; zero is a valid bit pattern.
        let mut device_info: SP_DEVINFO_DATA = unsafe { mem::zeroed() };
        device_info.cbSize = mem::size_of::<SP_DEVINFO_DATA>() as u32;

        // SAFETY: `device_info_set` is a valid HDEVINFO; `device_info` has `cbSize` set.
        let result = unsafe { SetupDiEnumDeviceInfo(self.0, index, &raw mut device_info) };

        if result == FALSE {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(ERROR_NO_MORE_ITEMS as i32) {
                return Ok(None);
            }
            return Err(err);
        }

        Ok(Some(DeviceInfo {
            data: device_info,
            set: self,
        }))
    }
}

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        // SAFETY: `self.0` was returned by `SetupDiGetClassDevsW` and is only destroyed here.
        unsafe {
            SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

pub struct DeviceInfo<'a> {
    data: SP_DEVINFO_DATA,
    set: &'a DeviceInfoSet,
}

impl DeviceInfo<'_> {
    /// Uninstalls the device represented by this `DeviceInfo`.
    /// https://learn.microsoft.com/en-us/windows/win32/api/newdev/nf-newdev-diuninstalldevice.
    pub fn uninstall_device(self) -> io::Result<()> {
        let mut needs_reboot: windows_sys::core::BOOL = 0;
        // SAFETY: `info.set.0` and `info.data`
        // are valid and belong to the same enumeration. `needs_reboot` is a writable BOOL.
        let result = unsafe {
            DiUninstallDevice(
                ptr::null_mut(),
                self.set.0,
                &raw const self.data,
                0,
                &raw mut needs_reboot,
            )
        };

        if result == FALSE {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

/// Enumerate devices of the given class.
pub struct DeviceInfoIter<'a> {
    index: u32,
    set: &'a DeviceInfoSet,
}

impl<'a> Iterator for DeviceInfoIter<'a> {
    type Item = io::Result<DeviceInfo<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.set.get_device_info(self.index).transpose()?;
        self.index += 1;
        Some(info)
    }
}

impl<'a> DeviceInfoIter<'a> {
    pub fn new(set: &'a DeviceInfoSet) -> Self {
        DeviceInfoIter { index: 0, set }
    }
}

/// Read the `NetCfgInstanceId` registry value from a device's driver key.
/// Returns the GUID string (e.g. `{AFE43773-...}`).
pub fn get_device_net_cfg_instance_id(info: &DeviceInfo<'_>) -> io::Result<String> {
    // SAFETY: `device_info_set` and `device_info`
    // are valid and belong to the same enumeration.
    let reg_key: HKEY = unsafe {
        SetupDiOpenDevRegKey(
            info.set.0,
            &raw const info.data,
            DICS_FLAG_GLOBAL,
            0,
            DIREG_DRV,
            KEY_READ,
        )
    };

    if std::ptr::eq(reg_key, INVALID_HANDLE_VALUE) {
        return Err(io::Error::last_os_error());
    }

    let mut buffer: Vec<u16> = vec![0u16; 128];
    let mut buffer_byte_len: u32 = (buffer.len() * 2) as u32;

    // SAFETY: `reg_key` is valid; `value_name` is NUL-terminated; `buffer` is sized by `buffer_byte_len`.
    let status = unsafe {
        RegGetValueW(
            reg_key,
            ptr::null(),
            w!("NetCfgInstanceId"),
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
