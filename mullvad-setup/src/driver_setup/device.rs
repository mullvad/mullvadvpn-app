use std::{
    ffi::{OsStr, OsString},
    io, mem,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    ptr,
};
use windows_sys::{
    Win32::{
        Devices::DeviceAndDriverInstallation::{
            DICD_GENERATE_ID, DICS_FLAG_GLOBAL, DIGCF_PRESENT, DIREG_DRV, DiUninstallDevice,
            HDEVINFO, SP_DEVINFO_DATA, SP_DRVINFO_DATA_V2_W, SP_DRVINFO_DETAIL_DATA_W,
            SPDIT_COMPATDRIVER, SPDRP_HARDWAREID, SUOI_FORCEDELETE, SetupDiBuildDriverInfoList,
            SetupDiCreateDeviceInfoListExW, SetupDiCreateDeviceInfoW, SetupDiDestroyDeviceInfoList,
            SetupDiDestroyDriverInfoList, SetupDiEnumDeviceInfo, SetupDiEnumDriverInfoW,
            SetupDiGetClassDevsW, SetupDiGetDeviceInfoListClass, SetupDiGetDeviceRegistryPropertyW,
            SetupDiGetDriverInfoDetailW, SetupDiOpenDevRegKey, SetupDiSetDeviceRegistryPropertyW,
            SetupUninstallOEMInfW,
        },
        Foundation::{ERROR_NO_MORE_ITEMS, FALSE, INVALID_HANDLE_VALUE},
        System::Registry::{HKEY, KEY_READ, RRF_RT_REG_SZ, RegCloseKey, RegGetValueW},
    },
    core::GUID,
};

use super::as_utf16_with_nul;

const ERROR_INSUFFICIENT_BUFFER: i32 =
    windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER as i32;

/// Type that represents a device information set. Uses [SetupDiGetClassDevsW].
///
/// [SetupDiGetClassDevsW]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetclassdevsw
pub struct DeviceInfoSet(HDEVINFO);

impl DeviceInfoSet {
    /// Return device information est for the given device class.
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

    /// Return an empty device information set for the given device class. Uses
    /// [`SetupDiCreateDeviceInfoListExW`].
    ///
    /// [`SetupDiCreateDeviceInfoListExW`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdicreatedeviceinfolistexw
    pub fn empty(class_guid: GUID) -> io::Result<Self> {
        // SAFETY: `class_guid` points to a valid GUID; the optional pointer args are NULL.
        let device_info_set = unsafe {
            SetupDiCreateDeviceInfoListExW(
                &raw const class_guid,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
            )
        };

        if device_info_set == INVALID_HANDLE_VALUE as isize {
            return Err(io::Error::last_os_error());
        }

        Ok(DeviceInfoSet(device_info_set))
    }

    /// Create a phantom device-info element in this set, in the set's class,
    /// with the given name. Uses [`SetupDiCreateDeviceInfoW`] with `DICD_GENERATE_ID`.
    ///
    /// [`SetupDiCreateDeviceInfoW`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdicreatedeviceinfow
    pub fn create_device(&self, device_name: &str) -> io::Result<DeviceInfo<'_>> {
        let class_guid = self.class_guid()?;
        let mut data = SP_DEVINFO_DATA {
            cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        let name_utf16 = as_utf16_with_nul(device_name);
        // SAFETY: `self.0` is a valid HDEVINFO; `name_utf16` is NUL-terminated UTF-16;
        // `class_guid` came from `SetupDiGetDeviceInfoListClass` on this set; `data.cbSize` is set.
        let ok = unsafe {
            SetupDiCreateDeviceInfoW(
                self.0,
                name_utf16.as_ptr(),
                &raw const class_guid,
                ptr::null(),
                ptr::null_mut(),
                DICD_GENERATE_ID,
                &raw mut data,
            )
        };
        if ok == FALSE {
            return Err(io::Error::last_os_error());
        }
        Ok(DeviceInfo { data, set: self })
    }

    /// Return the device setup class GUID this set is associated with (the one
    /// passed to `SetupDiGetClassDevsW` / `SetupDiCreateDeviceInfoListExW`).
    /// Uses [`SetupDiGetDeviceInfoListClass`].
    ///
    /// [`SetupDiGetDeviceInfoListClass`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinfolistclass
    fn class_guid(&self) -> io::Result<GUID> {
        let mut class_guid = GUID::default();
        // SAFETY: `self.0` is a valid HDEVINFO; `class_guid` is a writable GUID.
        let ok = unsafe { SetupDiGetDeviceInfoListClass(self.0, &raw mut class_guid) };
        if ok == FALSE {
            return Err(io::Error::last_os_error());
        }
        Ok(class_guid)
    }

    /// Return an iterator over the device info set.
    pub fn iter(&self) -> DeviceInfoIter<'_> {
        DeviceInfoIter::new(self)
    }

    fn get_device_info(&self, index: u32) -> io::Result<Option<DeviceInfo<'_>>> {
        let mut device_info = SP_DEVINFO_DATA {
            cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };

        // SAFETY: `self.0` is a valid HDEVINFO; `device_info` has `cbSize` set.
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
        // SAFETY: `self.0` was returned by `SetupDiGetClassDevsW` (or
        // `SetupDiCreateDeviceInfoListExW`) and is only destroyed here.
        unsafe {
            SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

/// Return the file names of every driver (in the driver store) that is
/// compatible with a device of class `class_guid` whose hardware ID is
/// `hardware_id`.
///
/// Uses the same SetupAPI dance as `WireGuardDeleteDriver`: build an empty device-info
/// list, add a phantom element with the desired hardware ID, then enumerate `SPDIT_COMPATDRIVER`.
pub fn list_compatible_driver_inf_names(
    class_guid: GUID,
    hardware_id: &str,
) -> io::Result<Vec<OsString>> {
    let set = DeviceInfoSet::empty(class_guid)?;
    let mut device = set.create_device(hardware_id)?;
    device.set_hardware_id(hardware_id)?;
    device.compat_driver_inf_names()
}

/// Uninstall an OEM INF from the driver store by its simple file name (e.g. `oem42.inf`).
/// Uses [`SetupUninstallOEMInfW`] with `SUOI_FORCEDELETE`.
///
/// [`SetupUninstallOEMInfW`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupuninstalloeminfw
pub fn uninstall_oem_inf(inf_name: &OsStr) -> io::Result<()> {
    let inf_filename_utf16 = as_utf16_with_nul(inf_name);
    // SAFETY: `inf_filename_utf16` is a NUL-terminated UTF-16 string; reserved must be null.
    let result = unsafe {
        SetupUninstallOEMInfW(inf_filename_utf16.as_ptr(), SUOI_FORCEDELETE, ptr::null())
    };
    if result == FALSE {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Type that represents a device information element from a [DeviceInfoSet].
///
/// See [`SetupDiEnumDeviceInfo`].
///
/// [`SetupDiEnumDeviceInfo`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdienumdeviceinfo
pub struct DeviceInfo<'a> {
    data: SP_DEVINFO_DATA,
    set: &'a DeviceInfoSet,
}

impl DeviceInfo<'_> {
    /// Uninstalls the device represented by this `DeviceInfo`. This calls [`DiUninstallDevice`].
    ///
    /// [`DiUninstallDevice`]: https://learn.microsoft.com/en-us/windows/win32/api/newdev/nf-newdev-diuninstalldevice.
    pub fn uninstall_device(self) -> io::Result<()> {
        let mut needs_reboot: windows_sys::core::BOOL = 0;
        // SAFETY: `self.set.0` and `self.data` are valid and belong to the same enumeration.
        // `needs_reboot` is a writable BOOL.
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

    /// Read the `NetCfgInstanceId` registry value from a device's driver key.
    /// Returns the GUID string (e.g. `{AFE43773-...}`).
    pub fn get_device_net_cfg_instance_id(&self) -> io::Result<String> {
        DevRegKey::open(self)?.get_string_value("NetCfgInstanceId")
    }

    /// Read the device's `HardwareID` property (`SPDRP_HARDWAREID`), which is a
    /// `REG_MULTI_SZ` list of hardware IDs.
    pub fn get_hardware_ids(&self) -> io::Result<Vec<String>> {
        let mut buffer = vec![0u16; 512];
        let mut required: u32 = 0;
        loop {
            // SAFETY: `self.set.0` and `self.data` are valid and belong to the same enumeration;
            // `buffer` is sized by its length.
            let result = unsafe {
                SetupDiGetDeviceRegistryPropertyW(
                    self.set.0,
                    &raw const self.data,
                    SPDRP_HARDWAREID,
                    ptr::null_mut(),
                    buffer.as_mut_ptr().cast(),
                    (2 * buffer.len()) as u32,
                    &raw mut required,
                )
            };
            let required = (required / 2) as usize; // bytes -> u16s
            if result != FALSE {
                buffer.truncate(required);
                break;
            }
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(ERROR_INSUFFICIENT_BUFFER) && required > buffer.len() {
                buffer.resize(required, 0);
                continue;
            }
            return Err(err);
        }

        // Split on NULs (REG_MULTI_SZ)
        Ok(buffer
            .split(|&c| c == 0)
            .filter(|s| !s.is_empty())
            .map(String::from_utf16_lossy)
            .collect())
    }

    /// Set the device's `HardwareID` property (`SPDRP_HARDWAREID`) to the
    /// single given hardware ID.
    ///
    /// # Panics
    ///
    /// This panics if `hardware_id.len()` is above `size_of::MAX / 2 - 2`.
    pub fn set_hardware_id(&mut self, hardware_id: &str) -> io::Result<()> {
        // SPDRP_HARDWAREID is REG_MULTI_SZ (NUL-delimited list), so terminate with two NULs.
        let multi_sz: Vec<u16> = hardware_id.encode_utf16().chain([0u16, 0u16]).collect();
        let byte_len = u32::try_from(multi_sz.len() * 2).expect("hardware ID size fits u32");
        // SAFETY: `self.set.0` and `self.data` are valid; `multi_sz` is a double-NUL-terminated
        // UTF-16 buffer of size `byte_len` bytes.
        let ok = unsafe {
            SetupDiSetDeviceRegistryPropertyW(
                self.set.0,
                &raw mut self.data,
                SPDRP_HARDWAREID,
                multi_sz.as_ptr().cast::<u8>(),
                byte_len,
            )
        };
        if ok == FALSE {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Build the list of drivers in the driver store compatible with this
    /// device (`SPDIT_COMPATDRIVER`) and return the simple file names of their
    /// INFs.
    ///
    /// [`SetupUninstallOEMInfW`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupuninstalloeminfw
    pub fn compat_driver_inf_names(&mut self) -> io::Result<Vec<OsString>> {
        CompatDriverInfoList::build(self)?.inf_names()
    }
}

/// A compatible-driver list (`SPDIT_COMPATDRIVER`) built for a [`DeviceInfo`].
struct CompatDriverInfoList<'d, 'a> {
    device: &'d mut DeviceInfo<'a>,
}

impl<'d, 'a: 'd> CompatDriverInfoList<'d, 'a> {
    /// Build a compatible driver list for the given `device` using [`SetupDiBuildDriverInfoList`].
    ///
    /// [`SetupDiBuildDriverInfoList`]: https://learn.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdibuilddriverinfolist
    pub fn build(device: &'d mut DeviceInfo<'a>) -> io::Result<Self> {
        // SAFETY: `device.set.0` and `device.data` are valid and belong to the same set.
        let ok = unsafe {
            SetupDiBuildDriverInfoList(device.set.0, &raw mut device.data, SPDIT_COMPATDRIVER)
        };
        if ok == FALSE {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { device })
    }

    /// List the filenames of each compatible driver's INF.
    pub fn inf_names(&self) -> io::Result<Vec<OsString>> {
        let mut inf_names = Vec::new();
        for idx in 0u32.. {
            let mut drvinfo_data = SP_DRVINFO_DATA_V2_W {
                cbSize: mem::size_of::<SP_DRVINFO_DATA_V2_W>() as u32,
                ..Default::default()
            };
            // SAFETY: `self.device.set.0`, `self.device.data`, and `drvinfo_data` are valid.
            let ok = unsafe {
                SetupDiEnumDriverInfoW(
                    self.device.set.0,
                    &raw const self.device.data,
                    SPDIT_COMPATDRIVER,
                    idx,
                    &raw mut drvinfo_data,
                )
            };
            if ok == FALSE {
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(ERROR_NO_MORE_ITEMS as i32) {
                    break;
                }
                return Err(err);
            }

            let mut detail = SP_DRVINFO_DETAIL_DATA_W {
                cbSize: mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as u32,
                ..Default::default()
            };
            let mut required: u32 = 0;
            // SAFETY: All pointer args are valid; `detail` is sized correctly via `cbSize`.
            let ok = unsafe {
                SetupDiGetDriverInfoDetailW(
                    self.device.set.0,
                    &raw const self.device.data,
                    &raw const drvinfo_data,
                    &raw mut detail,
                    mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as u32,
                    &raw mut required,
                )
            };
            // ERROR_INSUFFICIENT_BUFFER means the variable-length compatible-IDs tail did
            // not fit, but `InfFileName` lives in the fixed-size header and is populated.
            if ok == FALSE {
                let err = io::Error::last_os_error();
                if err.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER) {
                    return Err(err);
                }
            }

            let inf_path = detail
                .InfFileName
                .split(|&c| c == 0)
                .next()
                .expect("path is null-terminated");
            let inf_path = OsString::from_wide(inf_path);
            if let Some(file_name) = PathBuf::from(inf_path).file_name() {
                inf_names.push(file_name.to_os_string());
            }
        }

        Ok(inf_names)
    }
}

impl Drop for CompatDriverInfoList<'_, '_> {
    fn drop(&mut self) {
        // SAFETY: `self.device.set.0` and `self.device.data` are valid for as long as
        // this struct exists, and the driver info list was built with `SPDIT_COMPATDRIVER`.
        unsafe {
            SetupDiDestroyDriverInfoList(
                self.device.set.0,
                &raw const self.device.data,
                SPDIT_COMPATDRIVER,
            );
        }
    }
}

/// Enumerates devices for a particular [DeviceInfoSet].
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

struct DevRegKey {
    key: HKEY,
}

impl DevRegKey {
    /// Open the registry key for device-specific configuration.
    pub fn open(info: &DeviceInfo<'_>) -> io::Result<Self> {
        // SAFETY: The device info set and info are valid and belong to the same enumeration.
        let key = unsafe {
            SetupDiOpenDevRegKey(
                info.set.0,
                &raw const info.data,
                DICS_FLAG_GLOBAL,
                0,
                DIREG_DRV,
                KEY_READ,
            )
        };

        if std::ptr::eq(key, INVALID_HANDLE_VALUE) {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { key })
    }

    /// Read a `REG_SZ` type value from the driver key.
    pub fn get_string_value(&self, value_name: &str) -> io::Result<String> {
        let mut buffer: Vec<u16> = vec![0u16; 128];
        let mut buffer_byte_len: u32 = (buffer.len() * 2) as u32;

        let value_name = as_utf16_with_nul(value_name);

        // SAFETY: `self.key` is valid; `value_name` is NUL-terminated; `buffer` is sized by `buffer_byte_len`.
        let status = unsafe {
            RegGetValueW(
                self.key,
                ptr::null(),
                value_name.as_ptr(),
                RRF_RT_REG_SZ,
                ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &raw mut buffer_byte_len,
            )
        };

        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        let len = buffer
            .iter()
            .position(|&c| c == 0)
            .expect("RegGetValueW guarantees a NUL terminator for REG_SZ");
        Ok(String::from_utf16_lossy(&buffer[..len]))
    }
}

impl Drop for DevRegKey {
    fn drop(&mut self) {
        // SAFETY: `self.key` is a valid reg key handle
        unsafe { RegCloseKey(self.key) };
    }
}
