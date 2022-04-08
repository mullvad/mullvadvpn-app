use std::time::Duration;

#[cfg(target_os = "windows")]
mod inner {
    pub use std::time::Instant;
}

#[cfg(unix)]
#[path = "unix.rs"]
mod inner;

const MAX_SLEEP_INTERVAL: Duration = Duration::from_secs(60);

/// Represents a measurement of a monotonic clock.
/// Unlike [std::time::Instant], the difference between two
/// instances is guaranteed to include time spent in system
/// sleep.
#[derive(Clone, Copy)]
pub struct Instant {
    t: inner::Instant,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            t: inner::Instant::now(),
        }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.t.duration_since(earlier.t)
    }
}

/// Waits for the specified interval while taking into account system sleep or suspension.
/// The accuracy is to within about one minute.
pub async fn sleep(duration: Duration) {
    let started = Instant::now();

    loop {
        let elapsed = Instant::now().duration_since(started);

        if elapsed >= duration {
            return;
        }

        tokio::time::sleep(std::cmp::min(
            MAX_SLEEP_INTERVAL,
            duration.saturating_sub(elapsed),
        ))
        .await;
    }
}
