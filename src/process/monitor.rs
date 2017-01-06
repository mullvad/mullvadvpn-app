use clonablechild::{ClonableChild, ChildExt};

use process::OpenVpnBuilder;

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
pub trait MonitorChild: Clone + Send + 'static {
    /// Waits for the child to exit completely, returning if the child exited cleanly or not.
    fn wait(&self) -> io::Result<bool>;

    /// Forces the child to exit.
    fn kill(&self) -> io::Result<()>;

    /// Retreives the stdout stream for the child.
    fn stdout(&mut self) -> Option<ChildStdout>;

    /// Retreives the stderr stream for the child.
    fn stderr(&mut self) -> Option<ChildStderr>;
}

/// Trait for objects that can spawn any type of child process object implementing `MonitorChild`.
pub trait ChildSpawner<C: MonitorChild>: Send + 'static {
    /// Spawns the child process, returning a handle to it on success.
    fn spawn(&mut self) -> io::Result<C>;
}


impl MonitorChild for ClonableChild {
    fn wait(&self) -> io::Result<bool> {
        ClonableChild::wait(self).map(|exit_status| exit_status.success())
    }

    fn kill(&self) -> io::Result<()> {
        ClonableChild::kill(self)
    }

    fn stdout(&mut self) -> Option<ChildStdout> {
        self.stdout()
    }

    fn stderr(&mut self) -> Option<ChildStderr> {
        self.stderr()
    }
}

impl ChildSpawner<ClonableChild> for OpenVpnBuilder {
    fn spawn(&mut self) -> io::Result<ClonableChild> {
        OpenVpnBuilder::spawn(self).map(|child| child.into_clonable())
    }
}
