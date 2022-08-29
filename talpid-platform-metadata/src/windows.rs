use std::{
    ffi::OsString,
    io, iter,
    mem::{self, MaybeUninit},
    os::windows::ffi::OsStrExt,
};
use windows_sys::Win32::System::{
    LibraryLoader::{GetModuleHandleW, GetProcAddress},
    SystemInformation::OSVERSIONINFOW,
};

#[allow(non_camel_case_types)]
type RTL_OSVERSIONINFOW = OSVERSIONINFOW;

pub fn version() -> String {
    let (major, build) = WindowsVersion::new()
        .map(|version_info| {
            (
                version_info.windows_version_string(),
                version_info.build_number().to_string(),
            )
        })
        .unwrap_or_else(|_| ("N/A".to_string(), "N/A".to_string()));

    format!("Windows {} Build {}", major, build)
}

pub fn short_version() -> String {
    let version_string = WindowsVersion::new()
        .map(|version| version.windows_version_string())
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
        if ntdll == 0 {
            return Err(io::Error::last_os_error());
        }

        let function_address = unsafe { GetProcAddress(ntdll, b"RtlGetVersion\0" as *const u8) }
            .ok_or_else(|| io::Error::last_os_error())?;

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

    pub fn windows_version_string(&self) -> String {
        // Check https://en.wikipedia.org/wiki/List_of_Microsoft_Windows_versions#Personal_computer_versions 'Release version' column
        // for the correct NT versions for specific windows releases.
        match (self.major_version(), self.minor_version()) {
            (6, 1) => "7".into(),
            (6, 2) => "8".into(),
            (6, 3) => "8.1".into(),
            (10, 0) => {
                if self.build_number() < 22000 {
                    "10".into()
                } else {
                    "11".into()
                }
            }
            (major, minor) => format!("{}.{}", major, minor),
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
