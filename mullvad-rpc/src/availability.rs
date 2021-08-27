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
    pause_automatic: bool,
    offline: bool,
}

impl State {
    pub fn is_paused(&self) -> bool {
        self.pause_automatic
    }

    pub fn is_offline(&self) -> bool {
        self.offline
    }

    pub fn is_available(&self) -> bool {
        !self.is_paused() && !self.is_offline()
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
    pub fn pause(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.pause_automatic {
            state.pause_automatic = true;
            let _ = self.tx.send(*state);
        }
    }

    pub fn resume(&self) {
        let mut state = self.state.lock().unwrap();
        if state.pause_automatic {
            state.pause_automatic = false;
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

    pub fn wait_available(&self) -> impl Future<Output = Result<(), Error>> {
        self.wait_for_state(|state| state.is_available())
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
