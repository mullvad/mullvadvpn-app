use libc::{clock_gettime, clockid_t, timespec};
use std::{mem::MaybeUninit, time::Duration};

const NSEC_PER_SEC: i64 = 1000000000;

#[cfg(target_os = "macos")]
const CLOCK_ID: clockid_t = libc::CLOCK_MONOTONIC;

#[cfg(any(target_os = "linux", target_os = "android"))]
const CLOCK_ID: clockid_t = libc::CLOCK_BOOTTIME;

#[derive(Clone, Copy)]
pub struct Instant {
    t: timespec,
}

impl Instant {
    pub fn now() -> Self {
        Self { t: now() }
    }

    fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        // Assumptions:
        // * `tv_sec >= 0`
        // * `tv_nsec < NSEC_PER_SEC`

        let (tv_sec, tv_nsec) = if self.t.tv_nsec < earlier.t.tv_nsec {
            (
                self.t.tv_sec - earlier.t.tv_sec - 1,
                NSEC_PER_SEC - earlier.t.tv_nsec + self.t.tv_nsec,
            )
        } else {
            (
                self.t.tv_sec - earlier.t.tv_sec,
                self.t.tv_nsec - earlier.t.tv_nsec,
            )
        };

        if tv_sec < 0 {
            return None;
        }

        Some(Duration::new(tv_sec as _, tv_nsec as _))
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier)
            .unwrap_or(Duration::ZERO)
    }
}

fn now() -> timespec {
    let mut t = MaybeUninit::zeroed();
    unsafe {
        clock_gettime(CLOCK_ID, t.as_mut_ptr());
        t.assume_init()
    }
}
