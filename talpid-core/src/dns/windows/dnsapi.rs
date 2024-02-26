use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, OnceLock,
    },
    time::{Duration, Instant},
};
use windows_sys::Win32::Foundation::BOOL;

static FLUSH_TIMEOUT: Duration = Duration::from_secs(5);
static DNSAPI_HANDLE: OnceLock<DnsApi> = OnceLock::new();

const MAX_CONCURRENT_FLUSHES: usize = 5;

/// Errors that can happen when configuring DNS on Windows.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to flush the DNS cache.
    #[error("Call to flush DNS cache failed")]
    FlushCache,

    /// Too many flush attempts in progress.
    #[error("Too many flush attempts in progress")]
    TooManyFlushAttempts,

    /// Flushing the DNS cache timed out.
    #[error("Timeout while flushing DNS cache")]
    Timeout,
}

pub fn flush_resolver_cache() -> Result<(), Error> {
    DNSAPI_HANDLE.get_or_init(DnsApi::new).flush_cache()
}

struct DnsApi {
    in_flight_flush_count: Arc<AtomicUsize>,
}

impl DnsApi {
    fn new() -> Self {
        DnsApi {
            in_flight_flush_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn flush_cache(&self) -> Result<(), Error> {
        let update_flush_count_result =
            self.in_flight_flush_count
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                    if val >= MAX_CONCURRENT_FLUSHES {
                        return None;
                    }
                    Some(val + 1)
                });
        if update_flush_count_result.is_err() {
            return Err(Error::TooManyFlushAttempts);
        }

        let (tx, rx) = mpsc::channel();
        let flush_count = self.in_flight_flush_count.clone();

        std::thread::spawn(move || {
            let begin = Instant::now();

            let result = if unsafe { (DnsFlushResolverCache)() } != 0 {
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

#[link(name = "dnsapi")]
extern "system" {
    // Flushes the DNS resolver cache
    pub fn DnsFlushResolverCache() -> BOOL;
}
