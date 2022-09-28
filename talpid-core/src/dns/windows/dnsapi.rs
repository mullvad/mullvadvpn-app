use once_cell::sync::OnceCell;
use std::{
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};
use windows_sys::{
    w,
    Win32::{
        Foundation::BOOL,
        System::LibraryLoader::{
            FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
        },
    },
};

type FlushResolverCacheFn = unsafe extern "stdcall" fn() -> BOOL;

static DNSAPI_HANDLE: OnceCell<DnsApi> = OnceCell::new();
static FLUSH_TIMEOUT: Duration = Duration::from_secs(5);

const MAX_CONCURRENT_FLUSHES: usize = 5;

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

    /// Too many flush attempts in progress.
    #[error(display = "Too many flush attempts in progress")]
    TooManyFlushAttempts,

    /// Flushing the DNS cache timed out.
    #[error(display = "Timeout while flushing DNS cache")]
    Timeout,
}

pub fn flush_resolver_cache() -> Result<(), Error> {
    DNSAPI_HANDLE
        .get_or_try_init(|| DnsApi::new())?
        .flush_cache()
}

struct DnsApi {
    in_flight_flush_count: Arc<AtomicUsize>,
    flush_fn: FlushResolverCacheFn,
}

unsafe impl Send for DnsApi {}
unsafe impl Sync for DnsApi {}

impl DnsApi {
    fn new() -> Result<Self, Error> {
        let handle = unsafe { LoadLibraryExW(w!("dnsapi.dll"), 0, LOAD_LIBRARY_SEARCH_SYSTEM32) };
        if handle == 0 {
            return Err(Error::LoadDll(io::Error::last_os_error()));
        }

        let flush_fn = unsafe { GetProcAddress(handle, b"DnsFlushResolverCache\0" as *const u8) };
        let flush_fn = flush_fn.ok_or_else(|| {
            let error = io::Error::last_os_error();
            unsafe { FreeLibrary(handle) };
            Error::GetFunction(error)
        })?;

        Ok(DnsApi {
            in_flight_flush_count: Arc::new(AtomicUsize::new(0)),
            flush_fn: unsafe { *(&flush_fn as *const _ as *const _) },
        })
    }

    fn flush_cache(&self) -> Result<(), Error> {
        if self
            .in_flight_flush_count
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                if val >= MAX_CONCURRENT_FLUSHES {
                    return None;
                }
                Some(val + 1)
            })
            .is_err()
        {
            return Err(Error::TooManyFlushAttempts);
        }

        let (tx, rx) = mpsc::channel();
        let flush_count = self.in_flight_flush_count.clone();

        let flush_fn = self.flush_fn;

        std::thread::spawn(move || {
            let begin = Instant::now();

            let result = if unsafe { (flush_fn)() } != 0 {
                let elapsed = begin.elapsed();
                if elapsed >= FLUSH_TIMEOUT {
                    log::warn!(
                        "Flushing system DNS cache took {} seconds",
                        elapsed.as_secs()
                    );
                } else {
                    log::debug!("Flushed system DNS cache");
                }
                Ok(())
            } else {
                Err(Error::FlushCache)
            };
            let _ = tx.send(result);

            flush_count.fetch_sub(1, Ordering::SeqCst);
        });

        match rx.recv_timeout(FLUSH_TIMEOUT) {
            Ok(result) => result,
            Err(_timeout_err) => Err(Error::Timeout),
        }
    }
}
