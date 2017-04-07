use std::io;

use std::process::{ChildStderr, ChildStdout};

/// A module for monitoring child processes and get notified of events on them.
pub mod monitor;
use self::monitor::MonitoredChild;

/// A module for all OpenVPN related process management.
pub mod openvpn;

use clonablechild::ClonableChild;


impl MonitoredChild for ClonableChild {
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
