use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 100;

/// Pause background requests if [ApiAvailabilityHandle::reset_inactivity_timer] hasn't been
/// called for this long.
const INACTIVITY_TIME: Duration = Duration::from_secs(3 * 24 * 60 * 60);

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// The [`ApiAvailability`] instance was dropped, or the receiver lagged behind.
    #[error(display = "API availability instance was dropped")]
    Interrupted(#[error(source)] broadcast::error::RecvError),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct State {
    suspended: bool,
    pause_background: bool,
    offline: bool,
    inactive: bool,
}

impl State {
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    pub fn is_background_paused(&self) -> bool {
        self.offline || self.pause_background || self.suspended || self.inactive
    }

    pub fn is_offline(&self) -> bool {
        self.offline
    }
}

pub struct ApiAvailability {
    state: Arc<Mutex<State>>,
    tx: broadcast::Sender<State>,

    inactivity_timer: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl ApiAvailability {
    pub fn new(initial_state: State) -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        let state = Arc::new(Mutex::new(initial_state));

        let availability = ApiAvailability {
            state,
            tx,
            inactivity_timer: Arc::new(Mutex::new(None)),
        };
        availability.handle().reset_inactivity_timer();
        availability
    }

    pub fn get_state(&self) -> State {
        *self.state.lock().unwrap()
    }

    pub fn handle(&self) -> ApiAvailabilityHandle {
        ApiAvailabilityHandle {
            state: self.state.clone(),
            tx: self.tx.clone(),
            inactivity_timer: self.inactivity_timer.clone(),
        }
    }
}

impl Drop for ApiAvailability {
    fn drop(&mut self) {
        if let Some(timer) = self.inactivity_timer.lock().unwrap().take() {
            timer.abort();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiAvailabilityHandle {
    state: Arc<Mutex<State>>,
    tx: broadcast::Sender<State>,
    inactivity_timer: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl ApiAvailabilityHandle {
    /// Reset task that automatically pauses API requests due inactivity,
    /// starting it if it's not currently running.
    pub fn reset_inactivity_timer(&self) {
        log::trace!("Restarting API inactivity check");

        let self_ = self.clone();

        let mut inactivity_timer = self.inactivity_timer.lock().unwrap();
        if let Some(timer) = inactivity_timer.take() {
            timer.abort();
        }

        self.set_active();

        *inactivity_timer = Some(tokio::spawn(async move {
            talpid_time::sleep(INACTIVITY_TIME).await;
            self_.set_inactive();
        }));
    }

    /// Stops timer that pauses API requests due to inactivity.
    pub fn stop_inactivity_timer(&self) {
        log::trace!("Stopping API inactivity check");

        let mut inactivity_timer = self.inactivity_timer.lock().unwrap();
        if let Some(timer) = inactivity_timer.take() {
            timer.abort();
        }
        self.set_active();
    }

    fn inactivity_timer_running(&self) -> bool {
        self.inactivity_timer.lock().unwrap().is_some()
    }

    pub fn suspend(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.suspended {
            log::debug!("Suspending API requests");

            state.suspended = true;
            let _ = self.tx.send(*state);
        }
    }

    pub fn unsuspend(&self) {
        let mut state = self.state.lock().unwrap();
        if state.suspended {
            log::debug!("Unsuspending API requests");

            state.suspended = false;
            let _ = self.tx.send(*state);
        }
    }

    pub fn pause_background(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.pause_background {
            log::debug!("Pausing background API requests");

            state.pause_background = true;
            let _ = self.tx.send(*state);
        }
    }

    pub fn resume_background(&self) {
        if self.inactivity_timer_running() {
            self.reset_inactivity_timer();
        }

        let mut state = self.state.lock().unwrap();
        if state.pause_background {
            log::debug!("Resuming background API requests");
            state.pause_background = false;
            let _ = self.tx.send(*state);
        }
    }

    fn set_inactive(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.inactive {
            log::debug!("Pausing background API requests due to inactivity");
            state.inactive = true;
            let _ = self.tx.send(*state);
        }
    }

    fn set_active(&self) {
        let mut state = self.state.lock().unwrap();
        if state.inactive {
            log::debug!("Resuming background API requests due to activity");
            state.inactive = false;
            let _ = self.tx.send(*state);
        }
    }

    pub fn set_offline(&self, offline: bool) {
        let mut state = self.state.lock().unwrap();
        if state.offline != offline {
            if offline {
                log::debug!("Pausing API requests due to being offline");
            } else {
                log::debug!("Resuming API requests due to coming online");
            }

            state.offline = offline;
            let _ = self.tx.send(*state);
        }
    }

    pub fn get_state(&self) -> State {
        *self.state.lock().unwrap()
    }

    pub fn wait_for_unsuspend(&self) -> impl Future<Output = Result<(), Error>> {
        self.wait_for_state(|state| !state.is_suspended())
    }

    pub fn when_bg_resumes<F: Future<Output = O>, O>(&self, task: F) -> impl Future<Output = O> {
        let wait_task = self.wait_for_state(|state| !state.is_background_paused());
        async move {
            let _ = wait_task.await;
            task.await
        }
    }

    pub fn wait_background(&self) -> impl Future<Output = Result<(), Error>> {
        self.wait_for_state(|state| !state.is_background_paused())
    }

    pub fn when_online<F: Future<Output = O>, O>(&self, task: F) -> impl Future<Output = O> {
        let wait_task = self.wait_for_state(|state| !state.is_offline());
        async move {
            let _ = wait_task.await;
            task.await
        }
    }

    pub fn wait_online(&self) -> impl Future<Output = Result<(), Error>> {
        self.wait_for_state(|state| !state.is_offline())
    }

    fn wait_for_state(
        &self,
        state_ready: impl Fn(State) -> bool,
    ) -> impl Future<Output = Result<(), Error>> {
        let mut rx = self.tx.subscribe();
        let state = self.state.clone();

        async move {
            let current_state = { *state.lock().unwrap() };
            if state_ready(current_state) {
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
