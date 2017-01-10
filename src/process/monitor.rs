use std::error::Error;
use std::fmt;
use std::io;
use std::process::{ChildStdout, ChildStderr};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Trait for listeners of events coming from `ChildMonitor`. All methods come with default
/// implementations that does nothing. So it is possible to implement this trait and only implement
/// the events one care about.
pub trait MonitorEventListener: Send + 'static {
    /// Event called when an `ChildMonitor::start` has been processed. Given `result` indicate if
    /// the start succeeded or failed.
    fn started(&mut self, result: TransitionResult<(Option<ChildStdout>, Option<ChildStderr>)>) {
        drop(result);
    }

    /// Event called when an `ChildMonitor::stop` has been processed. Given `result` indicate if
    /// the monitor successfully went from running to stopping. Note that this does not mean the
    /// subprocess is now dead, only that the monitor has initiated stopping it. See `child_exited`
    /// for actual process exit.
    fn stopping(&mut self, result: TransitionResult<()>) {
        drop(result);
    }

    /// Event called when the process monitored by the `ChildMonitor` has exited. `clean`
    /// indicate if the process exited with a zero exit code or not.
    fn child_exited(&mut self, clean: bool) {
        drop(clean);
    }
}

// The default listener, not doing anything on any event.
struct NullListener;
impl MonitorEventListener for NullListener {}


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
pub trait ChildSpawner<C: MonitoredChild>: Send + 'static {
    /// Spawns the child process, returning a handle to it on success.
    fn spawn(&mut self) -> io::Result<C>;
}


/// Type alias for results of transitions in the `ChildMonitor` state machine.
pub type TransitionResult<T> = Result<T, TransitionError>;

/// Error type for transitions in the `ChildMonitor` state machine.
#[derive(Debug)]
pub enum TransitionError {
    /// The transition could not be made because the state machine was not in a state that could
    /// transition to the desired state.
    InvalidState,

    /// The transition failed because of an `io::Error`.
    IoError(io::Error),
}

impl From<io::Error> for TransitionError {
    fn from(error: io::Error) -> Self {
        TransitionError::IoError(error)
    }
}

impl fmt::Display for TransitionError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for TransitionError {
    fn description(&self) -> &str {
        match *self {
            TransitionError::InvalidState => "Invalid state for desired transition",
            TransitionError::IoError(..) => "Transition failed due to IO error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TransitionError::IoError(ref e) => Some(e),
            _ => None,
        }
    }
}

enum Event {
    Start(TransitionResult<(Option<ChildStdout>, Option<ChildStderr>)>),
    Stop(TransitionResult<()>),
    ChildExited(bool),
}

enum State<C: MonitoredChild> {
    Stopped,
    Running(RunningState<C>),
    Stopping(StoppingState<C>),
}

struct RunningState<C: MonitoredChild> {
    child: C,
}

struct StoppingState<C: MonitoredChild> {
    child: C,
}

// Messages sent internally between the `ChildMonitor`, `StateMachine` and the thread that monitors
// the child process.
enum MonitorMsg {
    AddListener(Box<MonitorEventListener>),
    Start,
    Stop,
    ChildExited(bool),
    Shutdown,
}

fn spawn_state_machine<C, B>(builder: B) -> Sender<MonitorMsg>
    where C: MonitoredChild,
          B: ChildSpawner<C>
{
    let state_machine = StateMachine::new(builder);
    let tx = state_machine.get_handle();
    thread::spawn(move || {
        state_machine.run();
    });
    tx
}

struct StateMachine<C: MonitoredChild, B: ChildSpawner<C>> {
    process_builder: B,
    tx: Sender<MonitorMsg>,
    rx: Receiver<MonitorMsg>,
    listener: Box<MonitorEventListener>,
    state: State<C>,
}

impl<C: MonitoredChild, B: ChildSpawner<C>> StateMachine<C, B> {
    pub fn new(process_builder: B) -> Self {
        let (tx, rx) = mpsc::channel();
        let state_machine = StateMachine {
            process_builder: process_builder,
            tx: tx,
            rx: rx,
            listener: Box::new(NullListener),
            state: State::Stopped,
        };
        state_machine
    }

    pub fn get_handle(&self) -> Sender<MonitorMsg> {
        self.tx.clone()
    }

    pub fn run(mut self) {
        while let Ok(msg) = self.rx.recv() {
            let event = match msg {
                MonitorMsg::AddListener(listener) => {
                    self.set_listener(listener);
                    None
                }
                MonitorMsg::Start => Some(Event::Start(self.start())),
                MonitorMsg::Stop => Some(Event::Stop(self.stop())),
                MonitorMsg::ChildExited(success) => {
                    self.state = State::Stopped;
                    Some(Event::ChildExited(success))
                }
                MonitorMsg::Shutdown => break,
            };
            if let Some(event) = event {
                self.notify_listener(event);
            }
        }
    }

    fn set_listener(&mut self, listener: Box<MonitorEventListener>) {
        self.listener = listener;
    }

    fn start(&mut self) -> TransitionResult<(Option<ChildStdout>, Option<ChildStderr>)> {
        if let State::Stopped = self.state {
            let mut child = self.process_builder.spawn()?;
            let io = (child.stdout(), child.stderr());
            self.state = State::Running(RunningState { child: child.clone() });
            self.start_monitor_thread(child);
            Ok(io)
        } else {
            Err(TransitionError::InvalidState)
        }
    }

    fn start_monitor_thread(&self, child: C) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let success = child.wait().unwrap_or(false);
            drop(tx.send(MonitorMsg::ChildExited(success)));
        });
    }

    fn stop(&mut self) -> TransitionResult<()> {
        let result = if let State::Running(ref mut running_state) = self.state {
            running_state.child
                .kill()
                .map(|_| running_state.child.clone())
                .map_err(|e| TransitionError::IoError(e))
        } else {
            Err(TransitionError::InvalidState)
        };
        match result {
            Ok(child) => {
                self.state = State::Stopping(StoppingState { child: child });
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn notify_listener(&mut self, event: Event) {
        match event {
            Event::Start(result) => {
                self.listener.started(result);
            }
            Event::Stop(result) => {
                self.listener.stopping(result);
            }
            Event::ChildExited(clean) => {
                self.listener.child_exited(clean);
            }
        }
    }
}

impl<C: MonitoredChild, B: ChildSpawner<C>> Drop for StateMachine<C, B> {
    fn drop(&mut self) {
        drop(self.stop())
    }
}

/// A child process monitor. Takes care of starting and monitoring a child process and sends
/// out events about it to a registered listener.
pub struct ChildMonitor {
    state_machine: Sender<MonitorMsg>,
}

impl ChildMonitor {
    /// Creates a new `ChildMonitor` that spawn processes with the given `builder`. The new
    /// `ChildMonitor` will be in the stopped state and not start any process until you call
    /// `start()`
    pub fn new<C: MonitoredChild, B: ChildSpawner<C>>(builder: B) -> Self {
        ChildMonitor { state_machine: spawn_state_machine(builder) }
    }

    /// Set the event listener to `listener`. Note that events might not show up on the new
    /// listener immediately after this call since the listener change message must be processed by
    /// the backend first.
    pub fn set_listener<L>(&self, listener: L)
        where L: MonitorEventListener
    {
        self.state_machine.send(MonitorMsg::AddListener(Box::new(listener))).unwrap();
    }

    /// Start the process to monitor. This will trigger an `MonitorEventListener::started` event.
    pub fn start(&self) {
        self.state_machine.send(MonitorMsg::Start).unwrap();
    }

    /// Stop the monitored process. This will trigger an `MonitorEventListener::stopped` event.
    pub fn stop(&self) {
        self.state_machine.send(MonitorMsg::Stop).unwrap();
    }
}

impl Drop for ChildMonitor {
    fn drop(&mut self) {
        self.state_machine
            .send(MonitorMsg::Shutdown)
            .expect("Internal error, not able to send Shutdown");
    }
}


#[cfg(test)]
mod child_monitor {
    use std::io;
    use std::process::{ChildStdout, ChildStderr};
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc::{self, Sender, Receiver};
    use std::thread;
    use std::time::Duration;
    use super::*;

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

    impl ChildSpawner<MockChild> for MockChildSpawner {
        fn spawn(&mut self) -> io::Result<MockChild> {
            self.spawn_result
                .clone()
                .ok_or(io::Error::new(io::ErrorKind::Other, "Mocking a failed process spawn"))
        }
    }

    #[derive(Debug)]
    enum MockEvent {
        Start(TransitionResult<(bool, bool)>),
        Stop(TransitionResult<()>),
        ChildExited(bool),
    }

    struct MockListener {
        tx: Sender<MockEvent>,
    }

    impl MockListener {
        pub fn new() -> (Self, Receiver<MockEvent>) {
            let (tx, rx) = mpsc::channel();
            let mock_listener = MockListener { tx: tx };
            (mock_listener, rx)
        }
    }

    impl MonitorEventListener for MockListener {
        fn started(&mut self,
                   result: TransitionResult<(Option<ChildStdout>, Option<ChildStderr>)>) {
            let result = result.map(|(a, b)| (a.is_some(), b.is_some()));
            drop(self.tx.send(MockEvent::Start(result)));
        }

        fn stopping(&mut self, result: TransitionResult<()>) {
            drop(self.tx.send(MockEvent::Stop(result)));
        }

        fn child_exited(&mut self, clean: bool) {
            drop(self.tx.send(MockEvent::ChildExited(clean)));
        }
    }

    /// Tries to recv a message from the given `$rx` for one second and tries to match it with the
    /// given expected value, `$expected`
    macro_rules! assert_event {
        ($rx:ident, $expected:pat) => {{
            let result = $rx.recv_timeout(Duration::new(1, 0));
            if let $expected = result {} else {
                let msg = stringify!($expected);
                panic!("Expected {}. Got {:?}", msg, result);
            }
        }}
    }

    #[test]
    fn normal_start() {
        let builder = MockChildSpawner::new(Some(MockChild::instant_exit()));
        let (listener, rx) = MockListener::new();
        let testee = ChildMonitor::new(builder);

        testee.set_listener(listener);
        testee.start();

        assert_event!(rx, Ok(MockEvent::Start(Ok(_))));
        assert_event!(rx, Ok(MockEvent::ChildExited(true)));
    }

    #[test]
    fn start_failed() {
        let builder = MockChildSpawner::new(None);
        let (listener, rx) = MockListener::new();
        let testee = ChildMonitor::new(builder);

        testee.set_listener(listener);
        testee.start();

        assert_event!(rx, Ok(MockEvent::Start(Err(TransitionError::IoError(_)))));
    }

    #[test]
    fn notifies_latest_multiple_listeners() {
        let builder = MockChildSpawner::new(Some(MockChild::instant_exit()));
        let (listener, rx) = MockListener::new();
        let (listener2, rx2) = MockListener::new();
        let testee = ChildMonitor::new(builder);

        testee.set_listener(listener);
        testee.set_listener(listener2);
        testee.start();

        assert_event!(rx2, Ok(MockEvent::Start(Ok(_))));
        assert_event!(rx, Err(mpsc::RecvTimeoutError::Disconnected));
    }

    #[test]
    fn normal_stop() {
        let builder = MockChildSpawner::new(Some(MockChild::alive_until_kill()));
        let (listener, rx) = MockListener::new();
        let testee = ChildMonitor::new(builder);

        testee.set_listener(listener);
        testee.start();
        testee.stop();

        drop(rx.recv_timeout(Duration::new(1, 0)));
        assert_event!(rx, Ok(MockEvent::Stop(Ok(()))));
        assert_event!(rx, Ok(MockEvent::ChildExited(true)));
    }
}
