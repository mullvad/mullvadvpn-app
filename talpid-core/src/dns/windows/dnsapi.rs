use once_cell::sync::OnceCell;
use std::{io, ptr, sync::mpsc, time::Duration};
use winapi::{
    shared::minwindef::{BOOL, FALSE},
    um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
};

type FlushResolverCacheFn = unsafe extern "stdcall" fn() -> BOOL;

static FLUSH_RESOLVER_CACHE: OnceCell<FlushResolverCacheFn> = OnceCell::new();
static FLUSH_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors that can happen when configuring DNS on Windows.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to load dnsapi.dll.
    #[error(display = "Failed to load dnsapi.dll")]
    LoadDll(#[error(source)] io::Error),

    /// Failed to obtain exported function.
    #[error(display = "Failed to obtain flush function")]
    GetFunction(#[error(source)] io::Error),

    /// Failed to flush the DNS cache.
    #[error(display = "Call to flush DNS cache failed")]
    FlushCache,

    /// Flushing the DNS cache timed out.
    #[error(display = "Timeout while flushing DNS cache")]
    Timeout,
}

pub fn flush_resolver_cache() -> Result<(), Error> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if tx.send(flush_resolver_cache_inner()).is_err() {
            log::warn!("Flushing DNS cache completed (delayed)");
        }
    });

    match rx.recv_timeout(FLUSH_TIMEOUT) {
        Ok(result) => result,
        // TODO: Can this be a cancelled safely?
        Err(_timeout_err) => Err(Error::Timeout),
    }
}

fn flush_resolver_cache_inner() -> Result<(), Error> {
    let flush_cache = FLUSH_RESOLVER_CACHE.get_or_try_init(|| {
        let handle = unsafe {
            LoadLibraryExW(
                b"d\0n\0s\0a\0p\0i\0.\0d\0l\0l\0\0\0" as *const u8 as *const u16,
                ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
        };
        if handle.is_null() {
            return Err(Error::LoadDll(io::Error::last_os_error()));
        }
        let function_addr =
            unsafe { GetProcAddress(handle, b"DnsFlushResolverCache\0" as *const _ as *const i8) };
        if function_addr.is_null() {
            let error = io::Error::last_os_error();
            unsafe { FreeLibrary(handle) };
            return Err(Error::GetFunction(error));
        }
        Ok(unsafe { *(&function_addr as *const _ as *const _) })
    })?;

    if unsafe { flush_cache() } == FALSE {
        return Err(Error::FlushCache);
    }
    Ok(())
}
