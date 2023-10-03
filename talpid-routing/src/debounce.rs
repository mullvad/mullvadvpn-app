#![allow(dead_code)]

use std::{
    sync::mpsc::{channel, RecvTimeoutError, Sender},
    time::{Duration, Instant},
};

/// BurstGuard is a wrapper for a function that protects that function from being called too many
/// times in a short amount of time. To call the function use `burst_guard.trigger()`, at that point
/// `BurstGuard` will wait for `buffer_period` and if no more calls to `trigger` are made then it
/// will call the wrapped function. If another call to `trigger` is made during this wait then it
/// will wait another `buffer_period`, this happens over and over until either
/// `longest_buffer_period` time has elapsed or until no call to `trigger` has been made in
/// `buffer_period`. At which point the wrapped function will be called.
pub struct BurstGuard {
    sender: Sender<BurstGuardEvent>,
}

enum BurstGuardEvent {
    Trigger,
    Shutdown(Sender<()>),
}

impl BurstGuard {
    pub fn new<F: Fn() + Send + 'static>(callback: F) -> Self {
        /// This is the period of time the `BurstGuard` will wait for a new trigger to be sent
        /// before it calls the callback.
        const BURST_BUFFER_PERIOD: Duration = Duration::from_millis(200);
        /// This is the longest period that the `BurstGuard` will wait from the first trigger till
        /// it calls the callback.
        const BURST_LONGEST_BUFFER_PERIOD: Duration = Duration::from_secs(2);

        let (sender, listener) = channel();
        std::thread::spawn(move || {
            // The `stop` implementation assumes that this thread will not call `callback` again
            // if the listener has been dropped.
            while let Ok(message) = listener.recv() {
                match message {
                    BurstGuardEvent::Trigger => {
                        let start = Instant::now();
                        loop {
                            match listener.recv_timeout(BURST_BUFFER_PERIOD) {
                                Ok(BurstGuardEvent::Trigger) => {
                                    if start.elapsed() >= BURST_LONGEST_BUFFER_PERIOD {
                                        callback();
                                        break;
                                    }
                                }
                                Ok(BurstGuardEvent::Shutdown(tx)) => {
                                    let _ = tx.send(());
                                    return;
                                }
                                Err(RecvTimeoutError::Timeout) => {
                                    callback();
                                    break;
                                }
                                Err(RecvTimeoutError::Disconnected) => {
                                    break;
                                }
                            }
                        }
                    }
                    BurstGuardEvent::Shutdown(tx) => {
                        let _ = tx.send(());
                        return;
                    }
                }
            }
        });
        Self { sender }
    }

    /// When `stop` returns an then the `BurstGuard` thread is guaranteed to not make any further
    /// calls to `callback`.
    pub fn stop(self) {
        let (sender, listener) = channel();
        // If we could not send then it means the thread has already shut down and we can return
        if self.sender.send(BurstGuardEvent::Shutdown(sender)).is_ok() {
            // We do not care what the result is, if it is OK it means the thread shut down, if
            // it is Err it also means it shut down.
            let _ = listener.recv();
        }
    }

    /// Stop without waiting for in-flight events to complete.
    pub fn stop_nonblocking(self) {
        let (sender, _listener) = channel();
        let _ = self.sender.send(BurstGuardEvent::Shutdown(sender));
    }

    /// Asynchronously trigger burst
    pub fn trigger(&self) {
        self.sender.send(BurstGuardEvent::Trigger).unwrap();
    }
}
