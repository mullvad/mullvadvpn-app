use std::time::Duration;

use talpid_future::retry::{ConstantInterval, ExponentialBackoff, Jittered};

#[repr(C)]
pub struct SwiftRetryStrategy(*mut RetryStrategy);

impl SwiftRetryStrategy {
    /// # Safety
    /// The pointer must be a pointing to a valid instance of a `Box<RetryStrategy>`.
    pub unsafe fn into_rust(self) -> RetryStrategy {
        *Box::from_raw(self.0)
    }
}

pub struct RetryStrategy {
    delays: RetryDelay,
    max_retries: usize,
}

impl RetryStrategy {
    pub fn delays(self) -> impl Iterator<Item = Duration> + Send {
        let Self {
            delays,
            max_retries,
        } = self;

        let delays: Box<dyn Iterator<Item = Duration> + Send> = match delays {
            RetryDelay::Never => Box::new(std::iter::empty()),
            RetryDelay::Constant(constant_delays) => Box::new(constant_delays.take(max_retries)),
            RetryDelay::Exponential(exponential_delays) => Box::new(
                exponential_delays
                    .take(max_retries)
                    .inspect(|d| inspect_delay(d)),
            ),
        };

        Jittered::jitter(delays)
    }
}

#[repr(C)]
pub enum RetryDelay {
    Never,
    Constant(ConstantInterval),
    Exponential(ExponentialBackoff),
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_retry_strategy_never() -> SwiftRetryStrategy {
    let retry_strategy = RetryStrategy {
        delays: RetryDelay::Never,
        max_retries: 0,
    };

    let ptr = Box::into_raw(Box::new(retry_strategy));
    return SwiftRetryStrategy(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_retry_strategy_constant(
    max_retries: usize,
    delay_sec: u64,
) -> SwiftRetryStrategy {
    let interval = Duration::from_secs(delay_sec);
    let retry_strategy = RetryStrategy {
        delays: RetryDelay::Constant(ConstantInterval::new(interval, Some(max_retries))),
        max_retries: 0,
    };
    let ptr = Box::into_raw(Box::new(retry_strategy));

    return SwiftRetryStrategy(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn mullvad_api_retry_strategy_exponential(
    max_retries: usize,
    initial_seconds: u64,
    factor: u32,
    max_delay: u64,
) -> SwiftRetryStrategy {
    let initial_delay = Duration::from_secs(initial_seconds);

    let backoff = ExponentialBackoff::new(initial_delay, factor)
        .max_delay(Some(Duration::from_secs(max_delay)));

    let retry_strategy = RetryStrategy {
        delays: RetryDelay::Exponential(backoff),
        max_retries,
    };

    let ptr = Box::into_raw(Box::new(retry_strategy));
    return SwiftRetryStrategy(ptr);
}

#[no_mangle]
extern "C" fn inspect_delay(delay: &Duration) {
    println!("Should sleep for {} seconds", delay.as_secs());
}
