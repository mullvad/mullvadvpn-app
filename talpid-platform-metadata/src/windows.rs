use std::{
    ffi::OsString,
    io, iter,
    mem::{self, MaybeUninit},
    os::windows::ffi::OsStrExt,
};
use windows_sys::Win32::{
    Foundation::{NTSTATUS, STATUS_SUCCESS},
    System::{
        LibraryLoader::{GetModuleHandleW, GetProcAddress},
        SystemInformation::OSVERSIONINFOEXW,
        SystemServices::VER_NT_WORKSTATION,
    },
};

#[allow(non_camel_case_types)]
type RTL_OSVERSIONINFOEXW = OSVERSIONINFOEXW;

pub fn version() -> String {
    let (major, build) = WindowsVersion::new()
        .map(|version_info| {
            (
                version_info.windows_version_string(),
                version_info.build_number().to_string(),
            )
        })
        .unwrap_or_else(|_| ("N/A".to_owned(), "N/A".to_owned()));

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
    inner: RTL_OSVERSIONINFOEXW,
}

impl WindowsVersion {
    pub fn new() -> Result<WindowsVersion, io::Error> {
        let module_name: Vec<u16> = OsString::from("ntdll")
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0u16))
            .collect();

        // SAFETY: module_name is a valid UTF-16/WTF-16 null-terminated string.
        let ntdll = unsafe { GetModuleHandleW(module_name.as_ptr()) };
        if ntdll == 0 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: ntdll is a valid pointer, RtlGetVersion is a valid null-terminated ANSI string.
        let function_address = unsafe { GetProcAddress(ntdll, b"RtlGetVersion\0" as *const u8) }
            .ok_or_else(io::Error::last_os_error)?;

        // SAFETY: We're correcting this function pointer to the ACTUAL type of RtlGetVersion.
        // https://learn.microsoft.com/en-us/windows/win32/devnotes/rtlgetversion
        let rtl_get_version = unsafe {
            mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "stdcall" fn(*mut RTL_OSVERSIONINFOEXW) -> NTSTATUS,
            >(function_address)
        };

        let mut version_info: RTL_OSVERSIONINFOEXW =
            // SAFETY: RTL_OSVERSIONINFOEXW is a C struct and can safely be zeroed.
            unsafe { MaybeUninit::zeroed().assume_init() };

        version_info.dwOSVersionInfoSize = mem::size_of_val(&version_info) as u32;

        // SAFETY:
        // - &mut version_info is a valid pointer.
        // - rtl_get_version was provided by GetProcAddress and should be valid.
        let status = unsafe { rtl_get_version(&mut version_info) };
        debug_assert_eq!(
            status, STATUS_SUCCESS,
            "RtlGetVersion always returns success"
        );

        Ok(WindowsVersion {
            inner: version_info,
        })
    }

    pub fn windows_version_string(&self) -> String {
        // `wProductType != VER_NT_WORKSTATION` implies that OS is Windows Server
        // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_osversioninfoexw
        // NOTE: This does not deduce which Windows Server version is running.
        if u32::from(self.inner.wProductType) != VER_NT_WORKSTATION {
            return "Server".to_owned();
        }

        match self.release_version() {
            (major, 0) => major.to_string(),
            (major, minor) => format!("{major}.{minor}"),
        }
    }

    /// Release version. E.g. `(10, 0)` for Windows 10, or `(8, 0)` for Windows 8.1.
    pub fn release_version(&self) -> (u32, u32) {
        // Check https://en.wikipedia.org/wiki/List_of_Microsoft_Windows_versions#Personal_computer_versions 'Release version' column
        // for the correct NT versions for specific windows releases.
        match (self.major_version(), self.minor_version()) {
            (6, 1) => (7, 0),
            (6, 2) => (8, 0),
            (6, 3) => (8, 1),
            (10, 0) => {
                if self.build_number() < 22000 {
                    (10, 0)
                } else {
                    (11, 0)
                }
            }
            (major, minor) => (major, minor),
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
