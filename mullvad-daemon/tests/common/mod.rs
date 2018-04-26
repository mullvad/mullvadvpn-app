#![allow(dead_code)]

#[cfg(unix)]
extern crate libc;
#[cfg(not(unix))]
extern crate mullvad_ipc_client;
extern crate os_pipe;

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use duct;

use self::os_pipe::{pipe, PipeReader};

#[cfg(unix)]
pub static DAEMON_EXECUTABLE_PATH: &str = "../target/debug/mullvad-daemon";

#[cfg(not(unix))]
pub static DAEMON_EXECUTABLE_PATH: &str = r"..\target\debug\mullvad-daemon.exe";

fn prepare_relay_list<T: AsRef<Path>>(path: T) {
    let path = path.as_ref();

    if !path.exists() {
        File::create(path)
            .expect("Failed to create relay list file")
            .write_all(b"{ \"countries\": [] }")
            .expect("Failed to write relay list");
    }
}

pub struct DaemonRunner {
    process: Option<duct::Handle>,
    output: Arc<Mutex<BufReader<PipeReader>>>,
}

impl DaemonRunner {
    pub fn spawn() -> Self {
        prepare_relay_list("../dist-assets/relays.json");

        let (reader, writer) = pipe().expect("Failed to open pipe to connect to daemon");
        let process = cmd!(DAEMON_EXECUTABLE_PATH, "-v", "--disable-log-to-file")
            .dir("..")
            .env("MULLVAD_CACHE_DIR", "./")
            .env("MULLVAD_RESOURCE_DIR", "./dist-assets")
            .stderr_to_stdout()
            .stdout_handle(writer)
            .start()
            .expect("Failed to start daemon");

        DaemonRunner {
            process: Some(process),
            output: Arc::new(Mutex::new(BufReader::new(reader))),
        }
    }

    pub fn assert_output(&mut self, pattern: &'static str, timeout: Duration) {
        let (tx, rx) = mpsc::channel();
        let stdout = self.output.clone();

        thread::spawn(move || {
            Self::wait_for_output(stdout, pattern);
            tx.send(()).expect("Failed to report search result");
        });

        rx.recv_timeout(timeout)
            .expect(&format!("failed to search for {:?}", pattern));
    }

    fn wait_for_output(output: Arc<Mutex<BufReader<PipeReader>>>, pattern: &str) {
        let mut output = output
            .lock()
            .expect("Another thread panicked while holding a lock to the process output");

        let mut line = String::new();

        while !line.contains(pattern) {
            line.clear();
            output
                .read_line(&mut line)
                .expect("Failed to read line from daemon stdout");
        }
    }

    #[cfg(unix)]
    fn request_clean_shutdown(&mut self, process: &mut duct::Handle) -> bool {
        use duct::unix::HandleExt;

        process.send_signal(libc::SIGTERM).is_ok()
    }

    #[cfg(not(unix))]
    fn request_clean_shutdown(&mut self, _: &mut duct::Handle) -> bool {
        use self::mullvad_ipc_client::DaemonRpcClient;

        if let Ok(mut rpc_client) = DaemonRpcClient::new() {
            rpc_client.shutdown().is_ok()
        } else {
            false
        }
    }
}

impl Drop for DaemonRunner {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            if self.request_clean_shutdown(&mut process) {
                let process = Arc::new(process);
                let wait_handle = process.clone();
                let (finished_tx, finished_rx) = mpsc::channel();

                thread::spawn(move || finished_tx.send(wait_handle.wait().map(|_| ())).unwrap());

                let has_finished = finished_rx
                    .recv_timeout(Duration::from_secs(5))
                    .map_err(|_| ())
                    .and_then(|result| result.map_err(|_| ()))
                    .is_ok();

                if !has_finished {
                    process.kill().unwrap();
                }
            } else {
                process.kill().unwrap();
            }
        }
    }
}
