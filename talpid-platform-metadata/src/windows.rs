use std::{
    ffi::OsString,
    io, iter,
    mem::{self, MaybeUninit},
    os::windows::ffi::OsStrExt,
    ptr,
};
use winapi::um::{
    libloaderapi::{GetModuleHandleW, GetProcAddress},
    winnt::RTL_OSVERSIONINFOW,
};

pub fn version() -> String {
    let (major, minor, build) = WindowsVersion::new()
        .map(|version_info| {
            (
                version_info.major_version().to_string(),
                version_info.minor_version().to_string(),
                version_info.build_number().to_string(),
            )
        })
        .unwrap_or_else(|_| ("N/A".to_string(), "N/A".to_string(), "N/A".to_string()));

    format!("Windows {}.{} Build {}", major, minor, build)
}

pub fn short_version() -> String {
    let version_string = WindowsVersion::new()
        .map(|version| version.major_version().to_string())
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

        let rtl_get_version: extern "stdcall" fn(*mut RTL_OSVERSIONINFOW) =
            unsafe { *(&function_address as *const _ as *const _) };

        let mut version_info: MaybeUninit<RTL_OSVERSIONINFOW> = mem::MaybeUninit::zeroed();
        unsafe {
            (*version_info.as_mut_ptr()).dwOSVersionInfoSize =
                mem::size_of_val(&version_info) as u32;
            rtl_get_version(version_info.as_mut_ptr());

            Ok(WindowsVersion {
                inner: version_info.assume_init(),
            })
        }
    }

    pub fn major_version(&self) -> u32 {
        self.inner.dwMajorVersion
    }

    pub fn minor_version(&self) -> u32 {
        self.inner.dwMinorVersion
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
