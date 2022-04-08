use std::time::Duration;

#[cfg(target_os = "windows")]
#[path = "std_instant.rs"]
mod inner;

#[cfg(unix)]
#[path = "unix.rs"]
mod inner;

pub use inner::Instant;

const MAX_SLEEP_INTERVAL: Duration = Duration::from_secs(60);

/// `sleep` function that includes time spent in system sleep or suspension.
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
