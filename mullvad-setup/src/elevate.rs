//! Helpers for restarting and elevating the current program to admin.
//! Using `ShellExecuteExW` rather than a manifest lets us hide the new window instead of opening
//! cmd without changing the subsystem to Windows.

use std::ffi::OsStr;
use std::io;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Security::*;
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

/// Return whether the current process is running as a privileged user
pub fn is_running_as_admin() -> io::Result<bool> {
    let mut admin_group: *mut SID = std::ptr::null_mut();
    let authority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };

    // SAFETY: `&mut admin_group` is a valid pointer
    if unsafe {
        AllocateAndInitializeSid(
            &authority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as _,
            DOMAIN_ALIAS_RID_ADMINS as _,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut admin_group as *mut *mut SID as *mut *mut _,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    let mut is_member = 0i32;
    // SAFETY: `admin_group` points to a valid SID, and `&mut is_member` points to a valid i32.
    let result = unsafe { CheckTokenMembership(0, admin_group as *mut _, &mut is_member) };

    // SAFETY: `admin_group` was successfully allocated with `AllocateAndInitializeSid`.
    unsafe { FreeSid(admin_group as *mut _) };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(is_member != 0)
}

/// Re-run the process with the same arguments using `ShellExecuteExW`, but as a privileged user.
/// Note that `stdout` and `stderr` will be lost.
pub fn rerun_as_admin() -> ! {
    let exe_path = std::env::current_exe().unwrap();
    let exe_path_w = to_wide_null(exe_path.as_os_str());

    let verb = "runas";
    let verb_w = to_wide_null(OsStr::new(verb));

    let args = args_to_quoted_args(std::env::args().skip(1));
    let args_w = to_wide_null(OsStr::new(&args));

    let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_NOCLOSEPROCESS;
    sei.lpVerb = verb_w.as_ptr();
    sei.lpFile = exe_path_w.as_ptr();
    sei.lpParameters = args_w.as_ptr();
    sei.nShow = SW_HIDE;

    // SAFETY: `sei` is a valid SHELLEXECUTEINFOW instance, and all of the pointers
    // are valid until `rerun_as_admin` returns.
    let success = unsafe { ShellExecuteExW(&mut sei) };
    if success == 0 {
        // SAFETY: The handle must be closed due to SEE_MASK_NOCLOSEPROCESS
        unsafe { CloseHandle(sei.hProcess) };

        std::process::exit(1);
    }

    // SAFETY: `sei.hProcess` is a valid handle
    unsafe {
        WaitForSingleObject(sei.hProcess, INFINITE);
    }

    let mut status = 1;

    // SAFETY: `sei.hProcess` has exited and is a valid handle
    unsafe { GetExitCodeProcess(sei.hProcess, &mut status) };

    // SAFETY: The handle must be closed due to SEE_MASK_NOCLOSEPROCESS
    unsafe { CloseHandle(sei.hProcess) };

    std::process::exit(status as _);
}

/// Convert `s` to a null-terminated widestring
fn to_wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

/// Convert argument into a space-delimited string of arguments.
/// Arguments that contain spaces or quotes are quoted, and quotes
/// within arguments are escaped.
fn args_to_quoted_args(args: impl IntoIterator<Item = String>) -> String {
    args.into_iter()
        .map(|arg| {
            if arg.contains(' ') || arg.contains('"') {
                format!("\"{}\"", arg.replace('"', "\\\""))
            } else {
                arg
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_args_to_quoted() {
        assert_eq!(
            args_to_quoted_args([
                "hello".to_string(),
                "hello world".to_string(),
                "\"test\"".to_string(),
            ]),
            r#"hello "hello world" "\"test\"""#,
        );
    }
}
