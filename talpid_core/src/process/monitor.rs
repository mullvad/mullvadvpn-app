use duct;

use std::io;
use std::process;
use std::sync::Arc;
use std::thread;

/// A child process monitor. Takes care of starting and monitoring a child process and calls the
/// listener on child exit. If the child is still running when a `ChildMonitor` instance goes out
/// of scope it will kill the child and wait for it to exit properly.
pub struct ChildMonitor {
    child: Arc<duct::Handle>,
    thread: Option<thread::JoinHandle<()>>,
}

impl ChildMonitor {
    /// Starts the child process and begins to monitor it. `on_exit` will be called as soon as the
    /// child process exits.
    pub fn start<L>(expression: &duct::Expression, mut on_exit: L) -> io::Result<Self>
        where L: FnMut(io::Result<&process::Output>) + Send + 'static
    {
        let child = Arc::new(expression.start()?);
        let child_clone = child.clone();
        let thread = Some(thread::spawn(move || on_exit(child_clone.wait())));
        Ok(ChildMonitor { child, thread })
    }

    /// Wait for the child to exit. Blocking the current thread. The `on_exit` callback is
    /// guaranteed to fire before this method returns.
    pub fn wait(&mut self) -> io::Result<&process::Output> {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.child.wait()
    }

    /// Send a kill signal to the child. No need to call `wait` after to free the PID. The monitor
    /// will wait for the process for you.
    pub fn kill(&self) -> io::Result<()> {
        self.child.kill()
    }
}

impl Drop for ChildMonitor {
    fn drop(&mut self) {
        let _ = self.kill();
        let _ = self.wait();
    }
}


#[cfg(test)]
mod child_monitor_tests {
    use super::*;
    use duct::{Expression, cmd};

    use std::sync::mpsc;
    use std::time::Duration;

    fn echo_cmd(s: &str) -> Expression {
        cmd("echo", &[s]).stdout_capture().unchecked()
    }

    fn invalid_cmd() -> Expression {
        cmd("this command does not exist", &[""]).unchecked()
    }

    fn sleep_cmd(secs: u32) -> Expression {
        if cfg!(windows) {
            cmd("ping", &["127.0.0.1", "-n", &(secs + 1).to_string()]).unchecked()
        } else {
            cmd("sleep", &[secs.to_string()]).unchecked()
        }
    }

    fn spawn(cmd: &Expression) -> (ChildMonitor, mpsc::Receiver<io::Result<process::Output>>) {
        let (tx, rx) = mpsc::channel();
        let child =
            ChildMonitor::start(cmd, move |res| tx.send(res.map(|out| out.clone())).unwrap())
                .expect("Unable to start process");
        (child, rx)
    }

    #[test]
    fn echo() {
        let (mut child, rx) = spawn(&echo_cmd("foobar"));
        let wait_output = child.wait().unwrap();
        let callback_output = rx.try_recv().unwrap().unwrap();

        assert!(callback_output.status.success());
        assert_eq!("foobar\n".as_bytes(), &callback_output.stdout[..]);
        assert_eq!(wait_output.status, callback_output.status);
        assert_eq!(wait_output.stdout, callback_output.stdout);
    }

    #[test]
    fn invalid_command() {
        assert!(ChildMonitor::start(&invalid_cmd(), |_| {}).is_err());
    }

    #[test]
    fn callback_after_kill() {
        let (child, rx) = spawn(&sleep_cmd(100000));
        // Make sure on_exit is not triggered within the first second. It should not be called
        // until we kill the process.
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());

        child.kill().unwrap();
        assert!(!rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap().status.success());
    }
}
