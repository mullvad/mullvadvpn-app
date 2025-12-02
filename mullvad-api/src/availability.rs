use std::{
    future::Future,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};
use tokio::sync::broadcast;

/// Pause background requests if [ApiAvailabilityHandle::reset_inactivity_timer] hasn't been
/// called for this long.
const INACTIVITY_TIME: Duration = Duration::from_hours(3 * 24);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The [`ApiAvailability`] instance was dropped, or the receiver lagged behind.
    #[error("API availability instance was dropped")]
    Interrupted(#[from] broadcast::error::RecvError),
}

#[derive(Clone, Debug)]
pub struct ApiAvailability(Arc<Mutex<ApiAvailabilityState>>);

#[derive(Debug)]
struct ApiAvailabilityState {
    tx: broadcast::Sender<State>,
    state: State,
    inactivity_timer: Option<tokio::task::JoinHandle<()>>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct State {
    suspended: bool,
    pause_background: bool,
    offline: bool,
    inactive: bool,
}

impl State {
    pub const fn is_suspended(&self) -> bool {
        self.suspended
    }

    pub const fn is_background_paused(&self) -> bool {
        self.offline || self.pause_background || self.suspended || self.inactive
    }

    pub const fn is_offline(&self) -> bool {
        self.offline
    }
}

impl ApiAvailability {
    const CHANNEL_CAPACITY: usize = 100;

    pub fn new(initial_state: State) -> Self {
        let (tx, _rx) = broadcast::channel(ApiAvailability::CHANNEL_CAPACITY);
        let inner = ApiAvailabilityState {
            state: initial_state,
            inactivity_timer: None,
            tx,
        };
        let handle = ApiAvailability(Arc::new(Mutex::new(inner)));
        // Start an inactivity timer
        handle.reset_inactivity_timer();
        handle
    }

    fn acquire(&self) -> MutexGuard<'_, ApiAvailabilityState> {
        self.0.lock().unwrap()
    }

    /// Reset task that automatically pauses API requests due inactivity,
    /// starting it if it's not currently running.
    pub fn reset_inactivity_timer(&self) {
        let mut inner = self.acquire();
        log::trace!("Restarting API inactivity check");
        inner.stop_inactivity_timer();
        let availability_handle = self.clone();
        inner.inactivity_timer = Some(tokio::spawn(async move {
            talpid_time::sleep(INACTIVITY_TIME).await;
            availability_handle.set_inactive();
        }));
        inner.set_active();
    }

    /// Stops timer that pauses API requests due to inactivity.
    pub fn stop_inactivity_timer(&self) {
        self.acquire().stop_inactivity_timer();
    }

    pub fn pause_background(&self) {
        self.acquire().pause_background();
    }

    pub fn resume_background(&self) {
        let should_reset = {
            let mut inner = self.acquire();
            inner.resume_background();
            inner.inactivity_timer_running()
        };
        // Note: It is important that we do not hold on to the Mutex when calling `reset_inactivity_timer()`.
        if should_reset {
            self.reset_inactivity_timer();
        }
    }

    pub fn suspend(&self) {
        self.acquire().suspend()
    }

    pub fn unsuspend(&self) {
        self.acquire().unsuspend();
    }

    pub fn set_offline(&self, offline: bool) {
        self.acquire().set_offline(offline);
    }

    fn set_inactive(&self) {
        self.acquire().set_inactive();
    }

    /// Check if the host is offline
    pub fn is_offline(&self) -> bool {
        self.get_state().is_offline()
    }

    fn get_state(&self) -> State {
        self.acquire().state
    }

    pub fn wait_for_unsuspend(&self) -> impl Future<Output = Result<(), Error>> + use<> {
        self.wait_for_state(|state| !state.is_suspended())
    }

    pub fn when_bg_resumes<F: Future<Output = O>, O>(
        &self,
        task: F,
    ) -> impl Future<Output = O> + use<F, O> {
        let wait_task = self.wait_for_state(|state| !state.is_background_paused());
        async move {
            let _ = wait_task.await;
            task.await
        }
    }

    pub fn wait_background(&self) -> impl Future<Output = Result<(), Error>> + use<> {
        self.wait_for_state(|state| !state.is_background_paused())
    }

    pub fn when_online<F: Future<Output = O>, O>(
        &self,
        task: F,
    ) -> impl Future<Output = O> + use<F, O> {
        let wait_task = self.wait_for_state(|state| !state.is_offline());
        async move {
            let _ = wait_task.await;
            task.await
        }
    }

    pub fn wait_online(&self) -> impl Future<Output = Result<(), Error>> {
        self.wait_for_state(|state| !state.is_offline())
    }

    fn wait_for_state<F: Fn(State) -> bool>(
        &self,
        state_ready: F,
    ) -> impl Future<Output = Result<(), Error>> + use<F> {
        let mut rx = { self.acquire().tx.subscribe() };

        let handle = self.clone();
        async move {
            let state = handle.get_state();
            if state_ready(state) {
                return Ok(());
            }

            loop {
                let new_state = rx.recv().await?;
                if state_ready(new_state) {
                    return Ok(());
                }
            }
        }
    }
}

impl Default for ApiAvailability {
    fn default() -> Self {
        ApiAvailability::new(State::default())
    }
}

impl ApiAvailabilityState {
    fn suspend(&mut self) {
        if !self.state.suspended {
            log::trace!("Suspending API requests");
            self.state.suspended = true;
            let _ = self.tx.send(self.state);
        }
    }

    fn unsuspend(&mut self) {
        if self.state.suspended {
            log::trace!("Unsuspending API requests");
            self.state.suspended = false;
            let _ = self.tx.send(self.state);
        }
    }

    fn set_inactive(&mut self) {
        log::trace!("Settings state to inactive");
        if !self.state.inactive {
            log::debug!("Pausing background API requests due to inactivity");
            self.state.inactive = true;
            let _ = self.tx.send(self.state);
        }
    }

    fn set_active(&mut self) {
        log::trace!("Settings state to active");
        if self.state.inactive {
            log::debug!("Resuming background API requests due to activity");
            self.state.inactive = false;
            let _ = self.tx.send(self.state).inspect_err(|send_err| {
                log::debug!("All receivers of state updates have been dropped");
                log::debug!("{send_err}");
            });
        }
    }

    fn set_offline(&mut self, offline: bool) {
        if offline {
            log::debug!("Pausing API requests due to being offline");
        } else {
            log::debug!("Resuming API requests due to coming online");
        }
        if self.state.offline != offline {
            self.state.offline = offline;
            let _ = self.tx.send(self.state);
        }
    }

    fn pause_background(&mut self) {
        if !self.state.pause_background {
            log::debug!("Pausing background API requests");
            self.state.pause_background = true;
            let _ = self.tx.send(self.state);
        }
    }

    fn resume_background(&mut self) {
        if self.state.pause_background {
            log::debug!("Resuming background API requests");
            self.state.pause_background = false;
            let _ = self.tx.send(self.state);
        }
    }

    fn stop_inactivity_timer(&mut self) {
        log::trace!("Stopping API inactivity check");
        if let Some(timer) = self.inactivity_timer.take() {
            timer.abort();
        }
    }

    const fn inactivity_timer_running(&self) -> bool {
        self.inactivity_timer.is_some()
    }
}

impl Drop for ApiAvailabilityState {
    fn drop(&mut self) {
        self.stop_inactivity_timer();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Use mockable time for tests
    pub use tokio::time::Duration;

    // Note that all of these tests needs a tokio runtime. Creating an instance of [`ApiAvailability`] will implicitly
    // spawn a tokio task.

    /// Test that the inactivity timer starts in an expected state.
    #[tokio::test(start_paused = true)]
    async fn test_initially_active() {
        // Start a new timer. It should *not* start as paused.
        let timer = ApiAvailability::default();
        assert!(
            !timer.get_state().is_background_paused(),
            "Inactivity timer should be active"
        )
    }

    /// Test that the inactivity timer kicks in after [`INACTIVITY_TIME`] of inactivity.
    #[tokio::test(start_paused = true)]
    async fn test_inactivity() {
        // Start a new timer. It should be marked as 'active'.
        let timer = ApiAvailability::default();
        // Elapse INACTIVITY_TIME (+ some slack because clocks)
        const SLACK: Duration = Duration::from_secs(1);
        talpid_time::sleep(INACTIVITY_TIME + SLACK).await;
        // Check that the timer is now marked as 'inactive'
        assert!(
            timer.get_state().is_background_paused(),
            "Inactivity timer should be inactive because 'INACTIVITY_TIME' has passed"
        )
    }
}
