use std::time::{Duration, Instant as StdInstant};

/// This implements functions similar to [std::time::Instant].
/// Unlike that type, this one is guaranteed to include time spent
/// in sleep/suspend.
#[derive(Clone, Copy)]
pub struct Instant {
    t: StdInstant,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            t: StdInstant::now(),
        }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.t.saturating_duration_since(earlier.t)
    }
}
