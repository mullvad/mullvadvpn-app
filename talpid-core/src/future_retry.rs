use rand::{distributions::OpenClosed01, Rng};
use std::{future::Future, time::Duration};

/// Since timers often exhibit weird behavior if they are running for too long, a workaround is
/// required - run a timer for 60 seconds until a delay is shorter than 5 minutes.
const MAX_SINGLE_DELAY: Duration = Duration::from_secs(5 * 60);

/// Retries a future until it should stop as determined by the retry function.
pub async fn retry_future_with_backoff<
    F: FnMut() -> O + 'static,
    R: FnMut(&T) -> bool + 'static,
    D: Iterator<Item = Duration> + 'static,
    O: Future<Output = T>,
    T,
>(
    mut factory: F,
    mut should_retry: R,
    mut delays: D,
) -> T {
    loop {
        let current_result = factory().await;
        if should_retry(&current_result) {
            if let Some(delay) = delays.next() {
                sleep(delay).await;
            }
        } else {
            return current_result;
        }
    }
}

async fn sleep(mut delay: Duration) {
    while delay > MAX_SINGLE_DELAY {
        delay -= MAX_SINGLE_DELAY;
        tokio::time::delay_for(MAX_SINGLE_DELAY).await;
    }

    tokio::time::delay_for(delay).await;
}

/// Provides an exponential back-off timer to delay the next retry of a failed operation.
pub struct ExponentialBackoff {
    current: u64,
    base: u64,
    factor: u64,
    max_delay: Option<Duration>,
}

impl ExponentialBackoff {
    /// Creates a `ExponentialBackoff` with the provided number of milliseconds as a base.
    ///
    /// All else staying the same, the first delay will be `millis` milliseconds long, the second
    /// one will be `millis^2`, third `millis^3` and so on.
    pub fn from_millis(millis: u64) -> ExponentialBackoff {
        ExponentialBackoff {
            current: millis,
            base: millis,
            factor: 1u64,
            max_delay: None,
        }
    }

    /// Sets the constant factor of the delays. The default value is 1.
    pub fn factor(mut self, factor: u64) -> ExponentialBackoff {
        self.factor = factor;
        self
    }

    /// Set the maximum delay. By default, there is no maximum value set, but the practical limit
    /// is `std::u64::MAX`.
    pub fn max_delay(mut self, duration: Duration) -> ExponentialBackoff {
        self.max_delay = Some(duration);
        self
    }

    /// Returns the value of the delay and advances the next back-off delay.
    fn next_delay(&mut self) -> Duration {
        let delay_msec = self
            .current
            .checked_mul(self.factor)
            .unwrap_or(std::u64::MAX);
        let delay = Duration::from_millis(delay_msec);

        if let Some(max_delay) = self.max_delay {
            if delay > max_delay {
                return max_delay;
            }
        }

        self.current = self.current.checked_mul(self.base).unwrap_or(std::u64::MAX);
        delay
    }

    /// Resets the delay to it's initial state.
    pub fn reset(&mut self) {
        self.current = self.base;
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        Some(self.next_delay())
    }
}

/// Adds jitter to a duration iterator
pub struct Jittered<I: Iterator<Item = Duration>> {
    inner: I,
}

impl<I: Iterator<Item = Duration>> Jittered<I> {
    /// Create an iterator of jittered durations
    pub fn jitter(inner: I) -> Self {
        Self { inner }
    }
}

impl<I: Iterator<Item = Duration>> Iterator for Jittered<I> {
    type Item = Duration;
    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.inner.next()?;
        Some(jitter(next_value))
    }
}

/// Apply a jitter to a duration.
fn jitter(dur: Duration) -> Duration {
    apply_jitter(dur, rand::thread_rng().sample(OpenClosed01))
}

fn apply_jitter(duration: Duration, jitter: f64) -> Duration {
    let secs = (duration.as_secs() as f64) * jitter;
    let nanos = (duration.subsec_nanos() as f64) * jitter;
    let millis = (secs * 1000f64) + (nanos / 1000000f64);
    Duration::from_millis(millis as u64)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_exponetnial_backoff() {
        let mut backoff = ExponentialBackoff::from_millis(2).factor(1000);

        assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(4)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(8)));
        backoff.reset();
        assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_at_maximum_value() {
        let mut backoff = ExponentialBackoff::from_millis(std::u64::MAX - 1);

        assert_eq!(
            backoff.next(),
            Some(Duration::from_millis(std::u64::MAX - 1))
        );
        assert_eq!(backoff.next(), Some(Duration::from_millis(std::u64::MAX)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(std::u64::MAX)));
    }

    #[test]
    fn test_maximum_bound() {
        let mut backoff = ExponentialBackoff::from_millis(2).max_delay(Duration::from_millis(4));

        assert_eq!(backoff.next(), Some(Duration::from_millis(2)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(4)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(4)));
    }

    #[test]
    fn test_minimum_value() {
        let mut backoff = ExponentialBackoff::from_millis(0);

        assert_eq!(backoff.next(), Some(Duration::from_millis(0)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(0)));
    }

    #[test]
    fn test_rounding() {
        let second = Duration::from_secs(1);
        assert_eq!(apply_jitter(second, 1.0), second);
    }

    #[quickcheck_macros::quickcheck]
    fn test_jitter(millis: u64, jitter: u64) {
        let max_num = 2u64.checked_pow(f64::MANTISSA_DIGITS).unwrap();
        let jitter = (jitter % max_num) as f64 / (max_num as f64);
        let unjittered_duration = Duration::from_millis(millis);
        let jittered_duration = apply_jitter(unjittered_duration, jitter);
        assert!(jittered_duration <= unjittered_duration);
    }
}
