#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use duct;
use duct::unix::HandleExt;
use libc;
use os_pipe::{pipe, PipeReader};

pub struct DaemonInstance {
    process: duct::Handle,
    output: Arc<Mutex<BufReader<PipeReader>>>,
}

impl DaemonInstance {
    pub fn new() -> Self {
        let (reader, writer) = pipe().expect("failed to open pipe to connect to daemon");
        let process = cmd!(
            "../target/debug/mullvad-daemon",
            "--resource-dir",
            "dist-assets"
        ).dir("..")
            .stderr_to_stdout()
            .stdout_handle(writer)
            .start()
            .expect("failed to start daemon");

        DaemonInstance {
            process,
            output: Arc::new(Mutex::new(BufReader::new(reader))),
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

    fn search_in_stdout(stdout: Arc<Mutex<BufReader<PipeReader>>>, pattern: &str) -> bool {
        if let Ok(mut output) = stdout.lock() {
            let mut line = String::new();

            while !line.contains(pattern) {
                output
                    .read_line(&mut line)
                    .expect("failed to read line from daemon stdout");
                println!("{}", line);
            }

            true
        } else {
            false
        }
    }
}

impl Drop for DaemonInstance {
    fn drop(&mut self) {
        let _ = self.process.send_signal(libc::SIGTERM);
    }
}
