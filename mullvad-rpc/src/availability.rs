use std::{
    future::Future,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;


const CHANNEL_CAPACITY: usize = 100;


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
}

impl State {
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    pub fn is_background_paused(&self) -> bool {
        self.offline || self.pause_background || self.suspended
    }

    pub fn is_offline(&self) -> bool {
        self.offline
    }
}

pub struct ApiAvailability {
    state: Arc<Mutex<State>>,
    tx: broadcast::Sender<State>,
}

impl ApiAvailability {
    pub fn new(initial_state: State) -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        let state = Arc::new(Mutex::new(initial_state));
        ApiAvailability { state, tx }
    }

    pub fn get_state(&self) -> State {
        *self.state.lock().unwrap()
    }

    pub fn handle(&self) -> ApiAvailabilityHandle {
        ApiAvailabilityHandle {
            state: self.state.clone(),
            tx: self.tx.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiAvailabilityHandle {
    state: Arc<Mutex<State>>,
    tx: broadcast::Sender<State>,
}

impl ApiAvailabilityHandle {
    pub fn suspend(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.suspended {
            state.suspended = true;
            let _ = self.tx.send(*state);
        }
    }

    pub fn unsuspend(&self) {
        let mut state = self.state.lock().unwrap();
        if state.suspended {
            state.suspended = false;
            let _ = self.tx.send(*state);
        }
    }

    pub fn pause_background(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.pause_background {
            state.pause_background = true;
            let _ = self.tx.send(*state);
        }
    }

    pub fn resume_background(&self) {
        let mut state = self.state.lock().unwrap();
        if state.pause_background {
            state.pause_background = false;
            let _ = self.tx.send(*state);
        }
    }

    pub fn set_offline(&self, offline: bool) {
        let mut state = self.state.lock().unwrap();
        if state.offline != offline {
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

    pub fn wait_background(&self) -> impl Future<Output = Result<(), Error>> {
        self.wait_for_state(|state| !state.is_background_paused())
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
