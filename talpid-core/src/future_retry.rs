use rand::{distributions::OpenClosed01, Rng};
use std::{future::Future, time::Duration};

/// Since timers often exhibit weird behavior if they are running for too long, a workaround is
/// required - run a timer for 60 seconds until a delay is shorter than 5 minutes.
const MAX_SINGLE_DELAY: Duration = Duration::from_secs(5 * 60);

/// Convenience function that works like [`retry_future`] but limits the number
/// of retries to `max_retries`.
pub async fn retry_future_n<
    F: FnMut() -> O + 'static,
    R: FnMut(&T) -> bool + 'static,
    D: Iterator<Item = Duration> + 'static,
    O: Future<Output = T>,
    T,
>(
    factory: F,
    should_retry: R,
    delays: D,
    max_retries: usize,
) -> T {
    retry_future(factory, should_retry, delays.take(max_retries)).await
}

/// Retries a future until it should stop as determined by the retry function, or when
/// the iterator returns `None`.
pub async fn retry_future<
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
                continue;
            }
        }
        return current_result;
    }
}

/// Returns an iterator that repeats the same interval.
pub fn constant_interval(interval: Duration) -> impl Iterator<Item = Duration> {
    std::iter::repeat(interval)
}

async fn sleep(mut delay: Duration) {
    while delay > MAX_SINGLE_DELAY {
        delay -= MAX_SINGLE_DELAY;
        tokio::time::sleep(MAX_SINGLE_DELAY).await;
    }

    tokio::time::sleep(delay).await;
}

/// Provides an exponential back-off timer to delay the next retry of a failed operation.
pub struct ExponentialBackoff {
    next: Duration,
    factor: u32,
    max_delay: Option<Duration>,
}

impl ExponentialBackoff {
    /// Creates a `ExponentialBackoff` starting with the provided duration.
    ///
    /// All else staying the same, the first delay will be `initial` long, the second
    /// one will be `initial * factor`, third `initial * factor^2` and so on.
    pub fn new(initial: Duration, factor: u32) -> ExponentialBackoff {
        ExponentialBackoff {
            next: initial,
            factor,
            max_delay: None,
        }
    }

    /// Set the maximum delay. By default, there is no maximum value set. The limit is
    /// `Duration::MAX`.
    pub fn max_delay(mut self, duration: Duration) -> ExponentialBackoff {
        self.max_delay = Some(duration);
        self
    }

    /// Returns the value of the delay and advances the next back-off delay.
    fn next_delay(&mut self) -> Duration {
        let next = self.next;

        if let Some(max_delay) = self.max_delay {
            if next > max_delay {
                return max_delay;
            }
        }

        self.next = next.saturating_mul(self.factor);

        next
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
    fn test_exponential_backoff() {
        let mut backoff = ExponentialBackoff::new(Duration::from_secs(2), 3);

        assert_eq!(backoff.next(), Some(Duration::from_secs(2)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(6)));
        assert_eq!(backoff.next(), Some(Duration::from_secs(18)));
    }

    #[test]
    fn test_at_maximum_value() {
        let max = Duration::MAX;
        let mu = Duration::from_micros(1);
        let mut backoff = ExponentialBackoff::new(max - mu, 2);

        assert_eq!(backoff.next(), Some(max - mu));
        assert_eq!(backoff.next(), Some(max));
        assert_eq!(backoff.next(), Some(max));
    }

    #[test]
    fn test_maximum_bound() {
        let mut backoff = ExponentialBackoff::new(Duration::from_millis(2), 3)
            .max_delay(Duration::from_millis(7));

        assert_eq!(backoff.next(), Some(Duration::from_millis(2)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(6)));
        assert_eq!(backoff.next(), Some(Duration::from_millis(7)));
    }

    #[test]
    fn test_minimum_value() {
        let zero = Duration::from_millis(0);
        let mut backoff = ExponentialBackoff::new(zero, 10);

        assert_eq!(backoff.next(), Some(zero));
        assert_eq!(backoff.next(), Some(zero));

        let mut backoff = ExponentialBackoff::new(Duration::from_millis(1), 0);

        assert_eq!(backoff.next(), Some(Duration::from_millis(1)));
        assert_eq!(backoff.next(), Some(zero));
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
