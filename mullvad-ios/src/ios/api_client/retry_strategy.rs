use std::time::Duration;

use talpid_future::retry::{ConstantInterval, ExponentialBackoff, Jittered};

#[derive(uniffi::Object, Clone, Copy)]
pub struct RetryStrategy {
    delays: RetryDelay,
}

impl RetryStrategy {
    pub fn delays(self) -> impl Iterator<Item = Duration> + Send {
        let Self { delays } = self;

        let delays: Box<dyn Iterator<Item = Duration> + Send> = match delays {
            RetryDelay::Never => Box::new(std::iter::empty()),
            RetryDelay::Constant {
                interval,
                max_retries,
            } => Box::new(ConstantInterval::new(
                Duration::from_secs(interval),
                Some(max_retries),
            )),
            RetryDelay::Exponential {
                initial_delay,
                factor,
                max_delay,
                max_retries,
            } => Box::new(
                ExponentialBackoff::new(Duration::from_secs(initial_delay), factor)
                    .max_delay(Some(Duration::from_secs(max_delay)))
                    .take(max_retries),
            ),
        };

        Jittered::jitter(delays)
    }
}

#[derive(Clone, Copy)]
pub enum RetryDelay {
    Never,
    Constant {
        interval: u64,
        max_retries: usize,
    },
    Exponential {
        initial_delay: u64,
        factor: u32,
        max_delay: u64,
        max_retries: usize,
    },
}

/// Creates a retry strategy that never retries after failure.
/// The result needs to be consumed.
#[uniffi::export]
pub fn mullvad_api_retry_strategy_never() -> RetryStrategy {
    RetryStrategy {
        delays: RetryDelay::Never,
    }
}

/// Creates a retry strategy that retries `max_retries` times with a constant delay of `delay_sec`.
/// The result needs to be consumed.
#[uniffi::export]
pub fn mullvad_api_retry_strategy_constant(max_retries: u64, delay_sec: u64) -> RetryStrategy {
    RetryStrategy {
        delays: RetryDelay::Constant {
            interval: delay_sec,
            max_retries: max_retries.try_into().unwrap_or(usize::MAX),
        },
    }
}

/// Creates a retry strategy that retries `max_retries` times with a exponantially increating delay.
/// The delay will never exceed `max_delay_sec`
/// The result needs to be consumed.
#[uniffi::export]
pub fn mullvad_api_retry_strategy_exponential(
    max_retries: u64,
    initial_sec: u64,
    factor: u32,
    max_delay_sec: u64,
) -> RetryStrategy {
    RetryStrategy {
        delays: RetryDelay::Exponential {
            initial_delay: initial_sec,
            factor,
            max_delay: max_delay_sec,
            max_retries: max_retries.try_into().unwrap_or(usize::MAX),
        },
    }
}
