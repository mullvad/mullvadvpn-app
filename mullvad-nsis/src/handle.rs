//! Functions for identifying and killing processes that interfere with updates.
//!
//! This is based on the [Restart Manager] API.
//!
//! [Restart Manager]: https://learn.microsoft.com/en-us/windows/win32/api/restartmanager/nf-restartmanager-rmstartsession

use anyhow::{Context, anyhow, bail};
use std::io;
use std::path::{Path, PathBuf};
use std::ptr;
use widestring::{WideCStr, WideCString, WideString, widestr};
use windows_sys::Win32::System::RestartManager::RmRebootReasonNone;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_OK, MB_TOPMOST};
use windows_sys::Win32::{
    Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
    System::RestartManager,
    UI::{
        Input::KeyboardAndMouse::GetActiveWindow,
        WindowsAndMessaging::{IDYES, MB_ICONWARNING, MB_YESNO, MessageBoxW},
    },
};

struct RMSession {
    handle: u32,
}

impl Drop for RMSession {
    fn drop(&mut self) {
        unsafe { RestartManager::RmEndSession(self.handle) };
        // NOTE: Ignoring error here
    }
}

/// Show a message box asking the user to close processes that prevent us from updating the files
/// in `dir_path`. If none are found, no message is shown.
///
/// On success, this returns whether to continue. If the user decides to cancel, it returns false.
///
/// This also returns false if Restart Manager refuses to continue without a reboot.
///
/// # Note
///
/// This function intentionally does not kill the Mullvad VPN service. The installer must manually
/// shut this down.
// TODO: Would it be better to keep the session active until it's been destroyed?
pub fn terminate_processes(
    dir_path: impl AsRef<Path>,
    allow_cancellation: bool,
) -> anyhow::Result<bool> {
    const MULLVAD_SERVICE: *const u16 = windows_sys::w!("mullvadvpn");

    // Text-Encoded session key defined in RestartManager.h
    // WCHAR sessKey[CCH_RM_SESSION_KEY+1];
    let mut strsessionkey: [u16; RestartManager::CCH_RM_SESSION_KEY as usize + 1] = [0; _];
    let mut handle = 0;

    let status =
        unsafe { RestartManager::RmStartSession(&raw mut handle, 0, strsessionkey.as_mut_ptr()) };
    if status != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(status as i32)).context("RmStartSession failed");
    }

    // End session when function returns
    let _session = RMSession { handle };

    let files = all_files_in_dir(dir_path)?;
    if files.is_empty() {
        // If the directory is empty, no user action is needed
        return Ok(true);
    }

    // Don't stop the Mullvad VPN service
    // TODO: consider letting RestartManager stopping the service
    let status = unsafe {
        RestartManager::RmAddFilter(
            handle,
            ptr::null(),
            ptr::null(),
            MULLVAD_SERVICE,
            RestartManager::RmNoShutdown,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(status as i32)).context("RmAddFilter failed");
    }

    // Encode files list as an array of pointers to wide c_strings
    let files: Vec<WideCString> = files
        .iter()
        .map(WideCString::from_os_str_truncate)
        .collect();
    let files: Vec<*const u16> = files.iter().map(|wcstr| wcstr.as_ptr()).collect();

    // Give RestartManager the list of files we are interested in.
    // SAFETY: `files` contains valid pointers into the shadowed `files`
    let status = unsafe {
        RestartManager::RmRegisterResources(
            handle,
            files.len() as u32,
            files.as_ptr(),
            0,
            ptr::null(),
            0,
            ptr::null(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(status as i32))
            .context("RmRegisterResources failed");
    }

    let mut n_proc_info_needed = 0;
    let mut n_affected_apps = 0;
    let mut reboot_reasons = RestartManager::RmRebootReasonNone;

    let mut rg_affected_apps =
        vec![RestartManager::RM_PROCESS_INFO::default(); n_affected_apps as usize];

    loop {
        let status = unsafe {
            RestartManager::RmGetList(
                handle,
                &raw mut n_proc_info_needed,
                &raw mut n_affected_apps,
                rg_affected_apps.as_mut_ptr(),
                (&raw mut reboot_reasons).cast::<u32>(),
            )
        };

        match status {
            ERROR_SUCCESS => {
                rg_affected_apps.truncate(n_affected_apps as usize);
                break;
            }
            ERROR_MORE_DATA => {
                n_affected_apps = n_proc_info_needed;
                rg_affected_apps.resize(n_affected_apps as usize, Default::default());
                continue;
            }
            err => {
                return Err(io::Error::from_raw_os_error(err as i32)).context("RmGetList failed");
            }
        }
    }

    if reboot_reasons != RmRebootReasonNone {
        bail!("Reboot required to continue");
    }

    // Ignore filtered apps
    rg_affected_apps
        .retain(|app| app.AppStatus as i32 & RestartManager::RmStatusShutdownMasked == 0);

    if rg_affected_apps.len() == 0 {
        // No apps need to be killed, so just continue
        return Ok(true);
    }

    if !ask_for_confirmation(&rg_affected_apps, allow_cancellation) {
        return Ok(false);
    }

    let result =
        unsafe { RestartManager::RmShutdown(handle, RestartManager::RmForceShutdown as _, None) };

    // TODO: Handle different results differently?
    // https://learn.microsoft.com/en-us/windows/win32/api/restartmanager/nf-restartmanager-rmshutdown

    if result == ERROR_SUCCESS {
        Ok(true)
    } else {
        Err(io::Error::from_raw_os_error(result as i32)).context("RmShutdown failed")
    }
}

/// Ask the user to confirm shutdown of the apps in `rg_affected_apps`.
/// This returns `true` if the user wishes to continue, `false` otherwise.
fn ask_for_confirmation(
    rg_affected_apps: &[RestartManager::RM_PROCESS_INFO],
    allow_cancellation: bool,
) -> bool {
    let mut s = WideString::new();
    for app in rg_affected_apps {
        let Ok(name) = WideCStr::from_slice_truncate(&app.strAppName) else {
            continue; // TODO: error?
        };
        s.push(name);
        s.push_char('\n');
    }

    if allow_cancellation {
        let mut message = widestr!("Some applications must be closed:\n\n").to_owned();
        message.push(s);
        message.push(widestr!("\nDo you wish to continue?"));
        let message = WideCString::from_ustr(message).expect("does not contain null");
        let result = unsafe {
            MessageBoxW(
                GetActiveWindow(),
                message.as_ptr(),
                windows_sys::w!("Applications must be closed"),
                MB_YESNO | MB_ICONWARNING | MB_TOPMOST,
            )
        };
        result == IDYES
    } else {
        let mut message = widestr!("Some applications must be closed:\n\n").to_owned();
        message.push(s);
        let message = WideCString::from_ustr(message).expect("does not contain null");
        unsafe {
            MessageBoxW(
                GetActiveWindow(),
                message.as_ptr(),
                windows_sys::w!("Applications must be closed"),
                MB_OK | MB_ICONWARNING | MB_TOPMOST,
            )
        };
        true
    }
}

/// Get the paths to all files in `path` and put them in `out`.
///
/// Symlinks are ignored.
fn all_files_in_dir(path: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    fn inner_all_files_in_dir(path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
        use std::fs;
        let path: &Path = path.as_ref();

        let dir_entries = fs::read_dir(path)?;
        for entry in dir_entries {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_file() {
                out.push(entry.path());
            } else if file_type.is_dir() {
                inner_all_files_in_dir(&entry.path(), out)?;
            } else if file_type.is_symlink() {
                // not implemented
            }
        }

        Ok(())
    }

    let path = path.as_ref();
    let mut out = vec![];
    inner_all_files_in_dir(path, &mut out)
        .with_context(|| anyhow!("Failed to recursively list files in {}", path.display()))?;
    Ok(out)
}

/// Check if a directory contains no files (only directories and symlinks).
/// Returns `true` if the directory exists and contains no files, only directories (or is empty).
/// Returns `false` if the directory contains any files.
pub fn is_empty_dir(path: impl AsRef<Path>) -> anyhow::Result<bool> {
    Ok(all_files_in_dir(path)?.is_empty())
}
