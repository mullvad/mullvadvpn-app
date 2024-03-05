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
    /// This is the period of time the `BurstGuard` will wait for a new trigger to be sent
    /// before it calls the callback.
    buffer_period: Duration,
    /// This is the longest period that the `BurstGuard` will wait from the first trigger till
    /// it calls the callback.
    longest_buffer_period: Duration,
}

enum BurstGuardEvent {
    Trigger(Duration),
    Shutdown(Sender<()>),
}

impl BurstGuard {
    pub fn new<F: Fn() + Send + 'static>(
        buffer_period: Duration,
        longest_buffer_period: Duration,
        callback: F,
    ) -> Self {
        let (sender, listener) = channel();
        std::thread::spawn(move || {
            // The `stop` implementation assumes that this thread will not call `callback` again
            // if the listener has been dropped.
            while let Ok(message) = listener.recv() {
                match message {
                    BurstGuardEvent::Trigger(mut period) => {
                        let start = Instant::now();
                        loop {
                            match listener.recv_timeout(period) {
                                Ok(BurstGuardEvent::Trigger(new_period)) => {
                                    period = new_period;
                                    let max_period = std::cmp::max(longest_buffer_period, period);
                                    if start.elapsed() >= max_period {
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
        Self {
            sender,
            buffer_period,
            longest_buffer_period,
        }
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
        talpid_types::detect_flood!();
        self.trigger_with_period(self.buffer_period)
    }

    /// Asynchronously trigger burst
    pub fn trigger_with_period(&self, buffer_period: Duration) {
        self.sender
            .send(BurstGuardEvent::Trigger(buffer_period))
            .unwrap();
    }
}
