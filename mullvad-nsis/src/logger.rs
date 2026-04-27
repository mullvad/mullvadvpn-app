//! Installer logging.
//!
//! Exports:
//! - `SetLogTarget` - open the log file (install.log / uninstall.log)
//! - `Log` - write a message to the log
//! - `LogWithDetails` - write a message with indented details
//! - `LogWindowsVersion` - log the Windows version string
//! - `GetWindowsMajorVersion` - push Windows major version onto the NSIS stack

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::ptr;
use std::sync::{Mutex, OnceLock};

use nsis_plugin_api::{nsis_fn, popint, pushint};
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW, LoadLibraryW,
};
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxA};

/// Global logger instance, protected by a mutex.
/// The log plugin pins itself with 100 LoadLibraryW calls to survive NSIS DLL unloads.
static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// Ensures the DLL is pinned exactly once.
static PINNED: OnceLock<()> = OnceLock::new();

/// Log target enumeration matching the NSIS script constants.
#[repr(i32)]
enum LogTarget {
    Install = 0,
    Uninstall = 1,
    Void = 2,
}

impl LogTarget {
    fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(LogTarget::Install),
            1 => Some(LogTarget::Uninstall),
            2 => Some(LogTarget::Void),
            _ => None,
        }
    }
}

/// A logger that writes timestamped UTF-8 messages to a file.
struct Logger {
    file: File,
}

impl Logger {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Logger { file })
    }

    fn log(&mut self, message: &str) {
        self.log_inner(&timestamp(), message);
    }

    fn log_with_details(&mut self, message: &str, details: &[&str]) {
        let ts = timestamp();
        self.log_inner(&ts, message);
        for detail in details {
            let detail_line = format!("{ts}     {detail}\r\n");
            let _ = self.file.write_all(detail_line.as_bytes());
        }
    }

    fn log_inner(&mut self, ts: &str, message: &str) {
        let line = format!("{ts} {message}\r\n");
        let _ = self.file.write_all(line.as_bytes());
    }
}

/// Format a local-time timestamp string in [YYYY-MM-DD HH:MM:SS.mmm] format.
fn timestamp() -> String {
    let mut time = SYSTEMTIME::default();
    // SAFETY: `&mut time` points to a stack-local SYSTEMTIME the API fills in.
    unsafe { GetLocalTime(&mut time) };

    let mut s = String::with_capacity(24);
    let _ = write!(
        s,
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}]",
        time.wYear,
        time.wMonth,
        time.wDay,
        time.wHour,
        time.wMinute,
        time.wSecond,
        time.wMilliseconds
    );
    s
}

/// Pin the DLL by incrementing its reference count 100 times.
/// NSIS unloads plugins between calls; pinning prevents this.
fn pin_dll() {
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
                &mut module,
            )
        };
        if success == 0 {
            return;
        }

        let mut path = [0u16; 260]; // MAX_PATH
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

// ============================================================================
// NSIS-exported functions
// ============================================================================

// SetLogTarget <target>
//
// Opens and maintains the log file handle.
// target: 0 = install.log, 1 = uninstall.log, 2 = void (disable logging)
#[nsis_fn]
fn SetLogTarget() -> Result<(), nsis_plugin_api::Error> {
    pin_dll();
    // SAFETY: `exdll_init` was called.
    let target_int = unsafe { popint()? };

    let result = (|| -> io::Result<()> {
        let target = LogTarget::from_i32(target_int);

        let mut logger = LOGGER.lock().unwrap_or_else(|e| e.into_inner());

        let logfile_name = match target {
            Some(LogTarget::Install) => "install.log",
            Some(LogTarget::Uninstall) => "uninstall.log",
            Some(LogTarget::Void) => {
                *logger = None;
                return Ok(());
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid log target",
                ));
            }
        };

        let program_data = mullvad_paths::windows::get_allusersprofile_dir()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let log_dir = program_data.join("Mullvad VPN");

        // Create the log directory with privileged access
        mullvad_paths::windows::create_privileged_directory(&log_dir)
            .map_err(|e| io::Error::other(format!("create log directory: {e}")))?;

        let log_path = log_dir.join(logfile_name);
        *logger = Some(Logger::new(log_path)?);

        Ok(())
    })();

    if let Err(e) = result {
        let msg = format!("Failed to set logging plugin target.\n{e}\0");
        // SAFETY: `msg` is a null-terminated ANSI string (the trailing `\0`
        // above), and the title argument is permitted to be null. `MB_OK`
        // is a valid flags value. `GetActiveWindow` returns a valid HWND or
        // null, both accepted by `MessageBoxA`.
        unsafe { MessageBoxA(GetActiveWindow(), msg.as_ptr(), ptr::null(), MB_OK) };
    }

    Ok(())
}

// Log "message"
//
// Writes a message to the log file.
#[nsis_fn]
fn Log() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: `exdll_init` was called.
    let message = unsafe { nsis_plugin_api::popstr()? };

    if let Ok(mut guard) = LOGGER.lock()
        && let Some(logger) = guard.as_mut()
    {
        logger.log(&message);
    }
    Ok(())
}

// LogWithDetails "message" "details"
//
// Writes a message followed by indented details to the log file.
// Details are newline-separated within a single NSIS string.
#[nsis_fn]
fn LogWithDetails() -> Result<(), nsis_plugin_api::Error> {
    // SAFETY: `exdll_init` was called.
    let (message, details) = unsafe { (nsis_plugin_api::popstr()?, nsis_plugin_api::popstr()?) };
    let detail_lines: Vec<&str> = details.lines().collect();

    if let Ok(mut guard) = LOGGER.lock()
        && let Some(logger) = guard.as_mut()
    {
        logger.log_with_details(&message, &detail_lines);
    }
    Ok(())
}

// LogWindowsVersion
//
// Logs the Windows version string.
#[nsis_fn]
fn LogWindowsVersion() -> Result<(), nsis_plugin_api::Error> {
    if let Ok(mut guard) = LOGGER.lock()
        && let Some(logger) = guard.as_mut()
    {
        let version = talpid_platform_metadata::version();
        logger.log(&format!("Windows version: {version}"));
    }
    Ok(())
}

// GetWindowsMajorVersion
//
// Pushes the Windows major version number onto the NSIS stack. Pushes -1 on error.
#[nsis_fn]
fn GetWindowsMajorVersion() -> Result<(), nsis_plugin_api::Error> {
    let value = match talpid_platform_metadata::WindowsVersion::from_ntoskrnl() {
        Ok(v) => v.major_version() as i32,
        Err(_) => {
            if let Ok(mut guard) = LOGGER.lock()
                && let Some(logger) = guard.as_mut()
            {
                logger.log("Windows version: Failed to determine version");
            }
            -1
        }
    };
    // SAFETY: `exdll_init` was called.
    unsafe { pushint(value) }
}
