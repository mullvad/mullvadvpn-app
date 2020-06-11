#![cfg(windows)]
use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, ptr};
use winapi::{
    shared::{minwindef::DWORD, winerror::ERROR_SUCCESS},
    um::{
        accctrl::*,
        aclapi::{SetEntriesInAclW, SetSecurityInfo},
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        securitybaseapi::{AllocateAndInitializeSid, FreeSid},
        winbase::LocalFree,
        winnt::*,
    },
};

struct Sid {
    sid_ptr: PSID,
}

impl Sid {
    pub fn new(authority: PSID_IDENTIFIER_AUTHORITY, relative_id: DWORD) -> Result<Sid, io::Error> {
        let mut sid = Sid {
            sid_ptr: ptr::null_mut(),
        };

        let result = unsafe {
            AllocateAndInitializeSid(
                authority,
                1,
                relative_id,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                &mut sid.sid_ptr as *mut _,
            )
        };
        if result != 0 {
            Ok(sid)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn as_ptr(&self) -> PSID {
        self.sid_ptr
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        unsafe { FreeSid(self.sid_ptr) };
    }
}

struct WinHandle(HANDLE);

impl WinHandle {
    pub fn get_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for WinHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

pub fn deny_network_access<T: AsRef<OsStr>>(ipc_path: T) -> Result<(), io::Error> {
    let mut ipc_w: Vec<_> = ipc_path.as_ref().encode_wide().collect();
    ipc_w.push(0u16);

    let pipe_handle = unsafe {
        CreateFileW(
            ipc_w.as_ptr(),
            GENERIC_WRITE | WRITE_DAC,
            0,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };

    if pipe_handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }

    let pipe_handle = WinHandle(pipe_handle);

    let network_sid = Sid::new(
        SECURITY_NT_AUTHORITY.as_mut_ptr() as *mut _,
        SECURITY_NETWORK_RID,
    )?;

    let mut network_access: EXPLICIT_ACCESS_W = unsafe { std::mem::zeroed() };
    network_access.grfAccessPermissions = GENERIC_READ | GENERIC_WRITE;
    network_access.grfAccessMode = DENY_ACCESS;
    network_access.grfInheritance = NO_INHERITANCE;
    network_access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    network_access.Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    network_access.Trustee.ptstrName = network_sid.as_ptr() as *mut _;

    let network_svc_sid = Sid::new(
        SECURITY_NT_AUTHORITY.as_mut_ptr() as *mut _,
        SECURITY_NETWORK_SERVICE_RID,
    )?;

    let mut network_svc_access: EXPLICIT_ACCESS_W = unsafe { std::mem::zeroed() };
    network_svc_access.grfAccessPermissions = GENERIC_READ | GENERIC_WRITE;
    network_svc_access.grfAccessMode = DENY_ACCESS;
    network_svc_access.grfInheritance = NO_INHERITANCE;
    network_svc_access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    network_svc_access.Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    network_svc_access.Trustee.ptstrName = network_svc_sid.as_ptr() as *mut _;

    let everyone_sid = Sid::new(
        SECURITY_WORLD_SID_AUTHORITY.as_mut_ptr() as *mut _,
        SECURITY_WORLD_RID,
    )?;

    let mut world_access: EXPLICIT_ACCESS_W = unsafe { std::mem::zeroed() };
    world_access.grfAccessPermissions = GENERIC_READ | GENERIC_WRITE;
    world_access.grfAccessMode = SET_ACCESS;
    world_access.grfInheritance = NO_INHERITANCE;
    world_access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    world_access.Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    world_access.Trustee.ptstrName = everyone_sid.as_ptr() as *mut _;

    let mut ace_entries = vec![network_access, network_svc_access, world_access];

    let mut new_dacl: PACL = unsafe { std::mem::zeroed() };
    let result = unsafe {
        SetEntriesInAclW(
            ace_entries.len() as u32,
            ace_entries.as_mut_ptr(),
            ptr::null_mut(),
            &mut new_dacl as *mut PACL,
        )
    };
    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    let result = unsafe {
        SetSecurityInfo(
            pipe_handle.get_raw(),
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            new_dacl as *mut ACL,
            ptr::null_mut(),
        )
    };

    unsafe { LocalFree(new_dacl as *mut _) };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    Ok(())
}
