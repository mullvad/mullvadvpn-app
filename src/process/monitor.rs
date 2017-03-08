use std::io;
use std::process::{ChildStdout, ChildStderr};
use std::sync::{Arc, Mutex};
use std::thread;


error_chain! {
    errors {
        /// The transition could not be made because the state machine was not in a state that
        /// could transition to the desired state.
        InvalidState {
            description("Invalid state for desired transition")
        }
        /// Error representing a failure in spawning the child process
        Spawn {
            description("Unable to spawn child process")
        }
        /// Error representing a failure in sending a kill signal to the child process
        Kill {
            description("Unable to send kill signal to process")
        }
    }
}

/// Trait for objects that represent child processes that `ChildMonitor` can monitor
pub trait MonitoredChild: Clone + Send + 'static {
    /// Waits for the child to exit completely, returning if the child exited cleanly or not.
    fn wait(&self) -> io::Result<bool>;

    /// Forces the child to exit.
    fn kill(&self) -> io::Result<()>;

    /// Retreives the stdout stream for the child.
    fn stdout(&mut self) -> Option<ChildStdout>;

    /// Retreives the stderr stream for the child.
    fn stderr(&mut self) -> Option<ChildStderr>;
}

/// Trait for objects that can spawn any type of child process object implementing `MonitoredChild`.
pub trait ChildSpawner: Send + 'static {
    /// The type of child being spawned.
    type Child: MonitoredChild;

    /// Spawns the child process, returning a handle to it on success.
    fn spawn(&mut self) -> io::Result<Self::Child>;
}


enum State<C: MonitoredChild> {
    Stopped,
    Running(RunningState<C>),
}

struct RunningState<C: MonitoredChild> {
    child: C,
    thread_handle: Option<thread::JoinHandle<()>>,
}

/// A child process monitor. Takes care of starting and monitoring a child process and runs the
/// listener on child exit.
pub struct ChildMonitor<S: ChildSpawner> {
    spawner: S,
    state: Arc<Mutex<State<S::Child>>>,
}

impl<S: ChildSpawner> ChildMonitor<S> {
    /// Creates a new `ChildMonitor` that spawns processes with the given `spawner`. The new
    /// `ChildMonitor` will be in the stopped state and not start any process until you call
    /// `start()`.
    pub fn new(spawner: S) -> Self {
        ChildMonitor {
            spawner: spawner,
            state: Arc::new(Mutex::new(State::Stopped)),
        }
    }

    /// Starts the child process and begins to monitor it. `listener` will be called as soon as the
    /// child process exits.
    pub fn start<L>(&mut self, listener: L) -> Result<(Option<ChildStdout>, Option<ChildStderr>)>
        where L: FnMut(bool) + Send + 'static
    {
        let mut state_lock = self.state.lock().unwrap();
        if let State::Stopped = *state_lock {
            let mut child = self.spawner.spawn().chain_err(|| ErrorKind::Spawn)?;
            let io = (child.stdout(), child.stderr());
            let thread_handle = self.spawn_monitor(child.clone(), listener);
            *state_lock = State::Running(RunningState {
                child: child,
                thread_handle: Some(thread_handle),
            });
            Ok(io)
        } else {
            Err(ErrorKind::InvalidState.into())
        }
    }

    fn spawn_monitor<L>(&self, child: S::Child, mut listener: L) -> thread::JoinHandle<()>
        where L: FnMut(bool) + Send + 'static
    {
        let state_mutex = self.state.clone();
        thread::spawn(move || {
            let success = child.wait().unwrap_or(false);
            {
                let mut state_lock = state_mutex.lock().unwrap();
                *state_lock = State::Stopped;
            }
            listener(success);
        })
    }

    /// Sends a kill signal to the child process.
    pub fn stop(&self) -> Result<()> {
        let state_lock = self.state.lock().unwrap();
        if let State::Running(ref running_state) = *state_lock {
            running_state.child.kill().chain_err(|| ErrorKind::Kill)?;
            Ok(())
        } else {
            Err(ErrorKind::InvalidState.into())
        }
    }
}

impl<S: ChildSpawner> Drop for ChildMonitor<S> {
    fn drop(&mut self) {
        let thread_handle = {
            let mut state_lock = self.state.lock().unwrap();
            if let State::Running(ref mut state) = *state_lock {
                let _ = state.child.kill();
                state.thread_handle.take()
            } else {
                None
            }
        };
        if let Some(thread_handle) = thread_handle {
            let _ = thread_handle.join();
        }
    }
}


#[cfg(test)]
mod child_monitor {
    use super::*;
    use std::io;
    use std::process::{ChildStdout, ChildStderr};
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[derive(Clone)]
    struct MockChild {
        died: Arc<Mutex<bool>>,
    }

    impl MockChild {
        pub fn instant_exit() -> Self {
            Self::new(true)
        }

        pub fn alive_until_kill() -> Self {
            Self::new(false)
        }

        fn new(died: bool) -> Self {
            MockChild { died: Arc::new(Mutex::new(died)) }
        }
    }

    impl MonitoredChild for MockChild {
        fn wait(&self) -> io::Result<bool> {
            loop {
                if *self.died.lock().unwrap() {
                    break;
                }
                thread::sleep(Duration::new(0, 1_000_000));
            }
            Ok(true)
        }

        fn kill(&self) -> io::Result<()> {
            *self.died.lock().unwrap() = true;
            Ok(())
        }

        fn stdout(&mut self) -> Option<ChildStdout> {
            None
        }

        fn stderr(&mut self) -> Option<ChildStderr> {
            None
        }
    }

    struct MockChildSpawner {
        spawn_result: Option<MockChild>,
    }

    impl MockChildSpawner {
        pub fn new(spawn_result: Option<MockChild>) -> Self {
            MockChildSpawner { spawn_result: spawn_result }
        }
    }

    impl ChildSpawner for MockChildSpawner {
        type Child = MockChild;

        fn spawn(&mut self) -> io::Result<MockChild> {
            self.spawn_result
                .clone()
                .ok_or(io::Error::new(io::ErrorKind::Other, "Mocking a failed process spawn"))
        }
    }

    /// Tries to recv a message from the given `$rx` for one second and tries to match it with the
    /// given expected value, `$expected`
    macro_rules! assert_event {
        ($rx:ident, $expected:pat) => {{
            let result = $rx.recv_timeout(Duration::new(1, 0));
            assert_matches!(result, $expected);
        }}
    }

    #[test]
    fn normal_start() {
        let spawner = MockChildSpawner::new(Some(MockChild::instant_exit()));
        let mut testee = ChildMonitor::new(spawner);

        let (tx, rx) = mpsc::channel();
        assert!(testee.start(move |success| tx.send(success).unwrap()).is_ok());
        assert_event!(rx, Ok(true));
    }

    #[test]
    fn start_failed() {
        let spawner = MockChildSpawner::new(None);
        let mut testee = ChildMonitor::new(spawner);

        let (tx, rx) = mpsc::channel();
        assert!(testee.start(move |success| tx.send(success).unwrap()).is_err());
        // Make sure that the listener is not kept anywhere. Failing to start should drop the
        // listener
        assert_event!(rx, Err(mpsc::RecvTimeoutError::Disconnected));
    }

    #[test]
    fn normal_stop() {
        let spawner = MockChildSpawner::new(Some(MockChild::alive_until_kill()));
        let mut testee = ChildMonitor::new(spawner);

        let (tx, rx) = mpsc::channel();
        assert!(testee.start(move |success| tx.send(success).unwrap()).is_ok());
        assert_event!(rx, Err(mpsc::RecvTimeoutError::Timeout));

        assert!(testee.stop().is_ok());
        assert_event!(rx, Ok(true));
    }

    #[test]
    fn stop_without_start() {
        let spawner = MockChildSpawner::new(Some(MockChild::alive_until_kill()));
        let testee = ChildMonitor::new(spawner);

        assert_matches!(testee.stop(), Err(Error(ErrorKind::InvalidState, _)));
    }
}
