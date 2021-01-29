use std::{ffi::OsString, io, iter, mem, os::windows::ffi::OsStrExt, ptr};
use winapi::um::{
    libloaderapi::{GetModuleHandleW, GetProcAddress},
    winnt::RTL_OSVERSIONINFOW,
};

pub fn version() -> String {
    short_version()
}

pub fn short_version() -> String {
    let version_string = WindowsVersion::new()
        .map(|version| {
            format!(
                "{} {}",
                version.major_version().to_string(),
                version.build_number().to_string()
            )
        })
        .unwrap_or("N/A".into());
    format!("Windows {}", version_string)
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    std::iter::empty()
}


pub struct WindowsVersion {
    inner: RTL_OSVERSIONINFOW,
}

impl WindowsVersion {
    pub fn new() -> Result<WindowsVersion, io::Error> {
        let rtl_get_version: extern "stdcall" fn(*mut RTL_OSVERSIONINFOW);

        let module_name: Vec<u16> = OsString::from("ntdll")
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0u16))
            .collect();

        let ntdll = unsafe { GetModuleHandleW(module_name.as_ptr()) };
        if ntdll == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }

        let function_address =
            unsafe { GetProcAddress(ntdll, b"RtlGetVersion\0" as *const _ as *const i8) };
        if function_address == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }

        rtl_get_version = unsafe { mem::transmute(function_address) };

        let mut version_info: RTL_OSVERSIONINFOW = unsafe { std::mem::zeroed() };
        version_info.dwOSVersionInfoSize = mem::size_of_val(&version_info) as u32;
        rtl_get_version(&mut version_info);

        Ok(WindowsVersion {
            inner: version_info,
        })
    }

    pub fn major_version(&self) -> u32 {
        self.inner.dwMajorVersion
    }

    pub fn build_number(&self) -> u32 {
        self.inner.dwBuildNumber
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_windows_version() {
        WindowsVersion::new().unwrap();
    }
}
