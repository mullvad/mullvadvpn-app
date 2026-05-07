use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::ptr;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::Foundation::{MAX_PATH, SYSTEMTIME};
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW, LoadLibraryW,
};
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

/// Global logger instance, protected by a mutex.
/// The log plugin pins itself with 100 LoadLibraryW calls to survive NSIS DLL unloads.
pub static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// Ensures the DLL is pinned exactly once.
static PINNED: OnceLock<()> = OnceLock::new();

/// Log target enumeration matching the NSIS script constants.
#[repr(i32)]
pub enum LogTarget {
    Install = 0,
    Uninstall = 1,
    Void = 2,
}

impl LogTarget {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(LogTarget::Install),
            1 => Some(LogTarget::Uninstall),
            2 => Some(LogTarget::Void),
            _ => None,
        }
    }
}

/// A logger that writes timestamped UTF-8 messages to a file.
pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Logger { file })
    }

    pub fn log(&mut self, message: &str) {
        let ts = timestamp();
        let line = format!("{ts} {message}\r\n");
        let _ = self.file.write_all(line.as_bytes());
    }

    pub fn log_with_details(&mut self, message: &str, details: &[&str]) {
        let ts = timestamp();
        let line = format!("{ts} {message}\r\n");
        let _ = self.file.write_all(line.as_bytes());
        for detail in details {
            let detail_line = format!("{ts}     {detail}\r\n");
            let _ = self.file.write_all(detail_line.as_bytes());
        }
    }
}

/// Format a local-time timestamp string in [YYYY-MM-DD HH:MM:SS.mmm] format.
fn timestamp() -> String {
    let mut time = SYSTEMTIME::default();
    // SAFETY: `&mut time` points to a stack-local SYSTEMTIME the API fills in.
    unsafe { GetLocalTime(&raw mut time) };
    format!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}]",
        time.wYear,
        time.wMonth,
        time.wDay,
        time.wHour,
        time.wMinute,
        time.wSecond,
        time.wMilliseconds
    )
}

/// Pin the DLL by incrementing its reference count 100 times.
/// NSIS unloads plugins between calls; pinning prevents this.
pub fn pin_dll() {
    PINNED.get_or_init(|| {
        let mut module: *mut std::ffi::c_void = ptr::null_mut();
        // SAFETY: `pin_dll` is a function in this DLL, so its address is a
        // valid lookup target with the FROM_ADDRESS flag. `&mut module` is
        // a stack-local for the API to fill in.
        let success = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                (pin_dll as *const ()).cast(),
                &raw mut module,
            )
        };
        if success == 0 {
            return;
        }

        let mut path = [0u16; MAX_PATH as usize];
        // SAFETY: `module` is a valid module handle returned above and `path`
        // is a writable buffer of `path.len()` u16s.
        let len = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32) };
        if len == 0 {
            return;
        }

        // Load 100 times so NSIS's FreeLibrary calls don't fully unload us
        for _ in 0..100 {
            // SAFETY: `path` is a null-terminated wide string written by
            // `GetModuleFileNameW` above (we exit early on `len == 0`).
            unsafe { LoadLibraryW(path.as_ptr()) };
        }
    });
}
