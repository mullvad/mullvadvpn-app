#![deny(rust_2018_idioms)]

use futures::Future;
use std::{io, thread};

use jsonrpc_core::{MetaIoHandler, Metadata};
use jsonrpc_ipc_server::{MetaExtractor, NoopExtractor, SecurityAttributes, Server, ServerBuilder};

use std::fmt;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use winapi::{
    shared::{minwindef::DWORD, winerror::ERROR_SUCCESS},
    um::{
        accctrl::*,
        aclapi::{GetSecurityInfo, SetEntriesInAclW, SetSecurityInfo},
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::{CloseHandle as WinCloseHandle, INVALID_HANDLE_VALUE},
        securitybaseapi::{AllocateAndInitializeSid, FreeSid},
        winbase::LocalFree,
        winnt::*,
    },
};

/// An Id created by the Ipc server that the client can use to connect to it
pub type IpcServerId = String;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Unable to start IPC server")]
    StartServerError(#[error(source)] io::Error),

    #[error(display = "IPC server thread panicked and never returned a start result")]
    ServerThreadPanicError,

    #[error(display = "Error in IPC server")]
    IpcServerError(#[error(source)] io::Error),

    #[error(display = "Unable to set permissions for IPC endpoint")]
    PermissionsError(#[error(source)] io::Error),
}


pub struct IpcServer {
    path: String,
    server: Server,
}

impl IpcServer {
    pub fn start<M: Metadata + Default>(
        handler: MetaIoHandler<M>,
        path: &str,
    ) -> Result<Self, Error> {
        Self::start_with_metadata(handler, NoopExtractor, path)
    }

    pub fn start_with_metadata<M, E>(
        handler: MetaIoHandler<M>,
        meta_extractor: E,
        path: &str,
    ) -> Result<Self, Error>
    where
        M: Metadata + Default,
        E: MetaExtractor<M>,
    {
        let security_attributes =
            SecurityAttributes::allow_everyone_create().map_err(Error::PermissionsError)?;
        let server = ServerBuilder::with_meta_extractor(handler, meta_extractor)
            .set_security_attributes(security_attributes)
            .start(path)
            .map_err(Error::StartServerError)
            .and_then(|(fut, start, server)| {
                thread::spawn(move || tokio::run(fut));
                if let Some(error) = start
                    .wait()
                    .map_err(|_cancelled| Error::ServerThreadPanicError)?
                {
                    return Err(Error::IpcServerError(error));
                }
                Ok(server)
            })
            .map(|server| IpcServer {
                path: path.to_owned(),
                server,
            })?;

        #[cfg(unix)]
        {
            use std::{fs, os::unix::fs::PermissionsExt};
            fs::set_permissions(&path, PermissionsExt::from_mode(0o766))
                .map_err(Error::PermissionsError)?;
        }
        #[cfg(windows)]
        deny_network_access(path).map_err(Error::PermissionsError)?;
        Ok(server)
    }

    /// Returns the uds/named pipe path this `IpcServer` is listening on.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Creates a handle bound to this `IpcServer` that can be used to shut it down.
    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle(self.server.close_handle())
    }

    /// Consumes the server and waits for it to finish. Get a `CloseHandle` before calling this
    /// if you want to be able to shut the server down.
    pub fn wait(self) {
        self.server.wait();
    }
}

// FIXME: This custom impl is because `Server` does not implement `Debug` yet:
// https://github.com/paritytech/jsonrpc/pull/195
impl fmt::Debug for IpcServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpcServer")
            .field("path", &self.path)
            .finish()
    }
}

#[derive(Clone)]
pub struct CloseHandle(jsonrpc_ipc_server::CloseHandle);

impl CloseHandle {
    pub fn close(self) {
        self.0.close();
    }
}

#[cfg(windows)]
struct Sid {
    sid_ptr: PSID,
}

#[cfg(windows)]
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

#[cfg(windows)]
impl Drop for Sid {
    fn drop(&mut self) {
        unsafe { FreeSid(self.sid_ptr) };
    }
}

#[cfg(windows)]
struct WinHandle(HANDLE);

#[cfg(windows)]
impl WinHandle {
    pub fn get_raw(&self) -> HANDLE {
        self.0
    }
}

#[cfg(windows)]
impl Drop for WinHandle {
    fn drop(&mut self) {
        unsafe { WinCloseHandle(self.0) };
    }
}

#[cfg(windows)]
fn deny_network_access<T: AsRef<OsStr>>(ipc_path: T) -> Result<(), io::Error> {
    let ipc_w: Vec<_> = ipc_path.as_ref().encode_wide().collect();

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

    let mut old_dacl: PACL = unsafe { std::mem::zeroed() };

    let result = unsafe {
        GetSecurityInfo(
            pipe_handle.get_raw(),
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut old_dacl as *mut PACL,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    if result != ERROR_SUCCESS {
        // A non-zero error code in WinError.h
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("GetSecurityInfo failed: {}", result),
        ));
    }

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
            old_dacl,
            &mut new_dacl as *mut PACL,
        )
    };
    if result != ERROR_SUCCESS {
        // A non-zero error code in WinError.h
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("SetEntriesInAclW failed: {}", result),
        ));
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
        // A non-zero error code in WinError.h
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("SetSecurityInfo failed: {}", result),
        ));
    }

    Ok(())
}
