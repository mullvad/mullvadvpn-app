//! Installer logging.

mod sys;

use sys::*;

use std::ptr;

use anyhow::{Context, bail};

use nsis_plugin_api::{nsis_fn, popint, pushint};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxA};

// SetLogTarget <target>
//
// Opens and maintains the log file handle.
// target: 0 = install.log, 1 = uninstall.log, 2 = void (disable logging)
#[nsis_fn]
fn SetLogTarget() -> Result<(), nsis_plugin_api::Error> {
    pin_dll();
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    let target_int = unsafe { popint()? };

    let result = (|| -> anyhow::Result<()> {
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
                bail!("Invalid log target");
            }
        };

        let log_dir = mullvad_paths::log_dir().context("Failed to get or create log dir")?;
        let log_path = log_dir.join(logfile_name);
        *logger = Some(Logger::new(log_path)?);

        Ok(())
    })();

    if let Err(e) = result {
        let msg = format!("Failed to set logging plugin target.\n{e:?}\0");
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
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
    // SAFETY: the `#[nsis_fn]` wrapper called `exdll_init` before this body
    // runs, initializing the static NSIS stack pointer.
    unsafe { pushint(value) }
}
