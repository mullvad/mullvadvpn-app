use std::{
    ffi::OsStr,
    io, iter,
    mem::{self, MaybeUninit},
    os::{raw::c_void, windows::ffi::OsStrExt},
    ptr,
};
use talpid_windows::env::get_system_dir;
use windows_sys::Win32::{
    Foundation::{NTSTATUS, STATUS_SUCCESS},
    Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FFI_SIGNATURE, VS_FIXEDFILEINFO,
        VerQueryValueW,
    },
    System::{
        LibraryLoader::{GetModuleHandleW, GetProcAddress},
        SystemInformation::OSVERSIONINFOEXW,
        SystemServices::VER_NT_WORKSTATION,
    },
};

#[allow(non_camel_case_types)]
type RTL_OSVERSIONINFOEXW = OSVERSIONINFOEXW;

pub fn version() -> String {
    let version =
        WindowsVersion::new().map(|v| (v.windows_version_string(), v.build_number().to_string()));
    let version_ntoskrnl = WindowsVersion::from_ntoskrnl()
        .map(|v| (v.windows_version_string(), v.build_number().to_string()));
    match (version, version_ntoskrnl) {
        (Ok(version), Ok(version_ntoskrnl)) if version == version_ntoskrnl => {
            format!("Windows {} Build {}", version.0, version.1)
        }
        // The Windows version given by ntdll and ntoskrnl are different.
        (Ok((major, build)), Ok((major_krnl, build_krnl))) => {
            format!(
                "Windows {major} Build {build}. Kernel version: Windows {major_krnl} Build {build_krnl}",
            )
        }
        (Ok((major, build)), _) | (_, Ok((major, build))) => {
            format!("Windows {major} Build {build}")
        }
        (Err(_), Err(_)) => "Windows N/A Build N/A".to_string(),
    }
}

pub fn short_version() -> String {
    let version_string = WindowsVersion::new()
        .map(|version| version.windows_version_string())
        .unwrap_or("N/A".into());
    format!("Windows {version_string}")
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    std::iter::empty()
}

pub struct WindowsVersion {
    major: u32,
    minor: u32,
    build: u32,
    product_type: ProductType,
}

#[derive(PartialEq)]
enum ProductType {
    Unknown,
    Workstation,
    Server,
}

impl WindowsVersion {
    pub fn new() -> Result<WindowsVersion, io::Error> {
        let module_name = to_wide("ntdll");

        // SAFETY: module_name is a valid UTF-16/WTF-16 null-terminated string.
        let ntdll = unsafe { GetModuleHandleW(module_name.as_ptr()) };
        if ntdll.is_null() {
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
                unsafe extern "system" fn(*mut RTL_OSVERSIONINFOEXW) -> NTSTATUS,
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
            major: version_info.dwMajorVersion,
            minor: version_info.dwMinorVersion,
            build: version_info.dwBuildNumber,
            product_type: match u32::from(version_info.wProductType) {
                // `wProductType != VER_NT_WORKSTATION` implies that OS is Windows Server
                // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_osversioninfoexw
                VER_NT_WORKSTATION => ProductType::Workstation,
                _ => ProductType::Server,
            },
        })
    }

    /// Extract Windows version information from the kernel image, which is unaffected by compatibility
    /// mode. Note that this does not infer whether we are running Windows Server or a normal version.
    pub fn from_ntoskrnl() -> io::Result<Self> {
        let (major, minor, build) = ntoskrnl_version()?;

        Ok(Self {
            major,
            minor,
            build,
            // NOTE: We do not have the product type here
            product_type: ProductType::Unknown,
        })
    }

    pub fn windows_version_string(&self) -> String {
        if self.product_type == ProductType::Server {
            // NOTE: This does not deduce which Windows Server version is running.
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
        self.major
    }

    pub fn minor_version(&self) -> u32 {
        self.minor
    }

    pub fn build_number(&self) -> u32 {
        self.build
    }
}

fn ntoskrnl_version() -> io::Result<(u32, u32, u32)> {
    let ntoskrnl_path = get_system_dir()?.join("ntoskrnl.exe");
    let wide_path = to_wide(ntoskrnl_path);
    let mut handle = 0u32;

    // SAFETY: We have a valid string and `handle` pointer
    let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
    if size == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut buffer = vec![0u8; size as usize];
    // SAFETY: `buffer` contains enough space to store the result
    let status =
        unsafe { GetFileVersionInfoW(wide_path.as_ptr(), 0, size, buffer.as_mut_ptr() as *mut _) };

    if status == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut lp_buffer: *mut c_void = ptr::null_mut();
    let mut len = 0u32;

    let sub_block = to_wide(r"\");
    // SAFETY: `buffer` points to a valid version-info resource
    let success = unsafe {
        VerQueryValueW(
            buffer.as_ptr() as *const _,
            sub_block.as_ptr(),
            &mut lp_buffer,
            &mut len,
        )
    };

    if success == 0 || lp_buffer.is_null() {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: `lp_buffer` points to a valid `VS_FIXEDFILEINFO`
    let info = unsafe { &*(lp_buffer as *const VS_FIXEDFILEINFO) };
    if info.dwSignature != VS_FFI_SIGNATURE as u32 {
        return Err(io::Error::other("Invalid version info signature"));
    }

    let major = info.dwProductVersionMS >> 16;
    let minor = info.dwProductVersionMS & 0xFFFF;
    let build = info.dwProductVersionLS >> 16;

    Ok((major, minor, build))
}

/// Return a null-terminated UTF16 string
fn to_wide(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(iter::once(0u16)).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_windows_version() {
        WindowsVersion::new().unwrap();
    }
}
