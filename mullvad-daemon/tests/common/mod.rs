use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct DaemonInstance {
    process: Child,
    output: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl DaemonInstance {
    pub fn new() -> Self {
        let mut process = Command::new("target/debug/mullvad-daemon")
            .stdout(Stdio::piped())
            .current_dir("..")
            .args(&["--resource-dir", "dist-assets"])
            .spawn()
            .expect("failed to start daemon");
        let output = BufReader::new(process.stdout.take().expect("missing daemon stdout"));

        DaemonInstance {
            process,
            output: Arc::new(Mutex::new(output)),
        }
    }

    pub fn output(&mut self) -> String {
        let mut bytes = Vec::new();
        self.output
            .lock()
            .expect("failed to access daemon stdout")
            .read_to_end(&mut bytes)
            .expect("failed to read daemon stdout");
        String::from_utf8_lossy(&bytes).to_string()
    }

    pub fn assert_log_contains(&mut self, pattern: &'static str, timeout: Duration) {
        let (tx, rx) = mpsc::channel();
        let stdout = self.output.clone();

        thread::spawn(move || {
            if Self::search_in_stdout(stdout, pattern) {
                tx.send(()).expect("failed to report search result");
            }
        });

        rx.recv_timeout(timeout)
            .expect(&format!("failed to search for {:?}", pattern));
    }

    fn search_in_stdout(stdout: Arc<Mutex<BufReader<ChildStdout>>>, pattern: &str) -> bool {
        if let Ok(mut output) = stdout.lock() {
            let mut line = String::new();

            while !line.contains(pattern) {
                output
                    .read_line(&mut line)
                    .expect("failed to read line from daemon stdout");
            }

            true
        } else {
            false
        }
    }
}

impl Drop for DaemonInstance {
    fn drop(&mut self) {
        self.process.kill().expect("failed to kill daemon process");
    }
}
