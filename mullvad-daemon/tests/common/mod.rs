#![allow(dead_code)]

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use duct;
use os_pipe::{pipe, PipeReader};

pub use self::platform_specific::*;

#[cfg(unix)]
mod platform_specific {
    use super::*;

    pub static DAEMON_EXECUTABLE_PATH: &str = "../target/debug/mullvad-daemon";

    pub fn rpc_file_path() -> PathBuf {
        Path::new("/tmp/.mullvad_rpc_address").to_path_buf()
    }
}

#[cfg(not(unix))]
mod platform_specific {
    use super::*;

    pub static DAEMON_EXECUTABLE_PATH: &str = r"..\target\debug\mullvad-daemon.exe";

    pub fn rpc_file_path() -> PathBuf {
        ::std::env::temp_dir().join(".mullvad_rpc_address")
    }
}

fn prepare_relay_list<T: AsRef<Path>>(path: T) {
    let path = path.as_ref();

    if !path.exists() {
        File::create(path)
            .expect("failed to create relay list file")
            .write_all(b"{ \"countries\": [] }")
            .expect("failed to write relay list");
    }
}

pub struct DaemonRunner {
    process: duct::Handle,
    output: Arc<Mutex<BufReader<PipeReader>>>,
}

impl DaemonRunner {
    pub fn spawn() -> Self {
        prepare_relay_list("../dist-assets/relays.json");

        let (reader, writer) = pipe().expect("failed to open pipe to connect to daemon");
        let process = cmd!(
            DAEMON_EXECUTABLE_PATH,
            "-v",
            "--resource-dir",
            "dist-assets"
        ).dir("..")
            .stderr_to_stdout()
            .stdout_handle(writer)
            .start()
            .expect("failed to start daemon");

        DaemonRunner {
            process,
            output: Arc::new(Mutex::new(BufReader::new(reader))),
        }
    }

    pub fn assert_output(&mut self, pattern: &'static str, timeout: Duration) {
        let (tx, rx) = mpsc::channel();
        let stdout = self.output.clone();

        thread::spawn(move || {
            Self::wait_for_output(stdout, pattern);
            tx.send(()).expect("failed to report search result");
        });

        rx.recv_timeout(timeout)
            .expect(&format!("failed to search for {:?}", pattern));
    }

    fn wait_for_output(output: Arc<Mutex<BufReader<PipeReader>>>, pattern: &str) {
        let mut output = output
            .lock()
            .expect("another thread panicked while holding a lock to the process output");

        let mut line = String::new();

        while !line.contains(pattern) {
            line.clear();
            output
                .read_line(&mut line)
                .expect("failed to read line from daemon stdout");
        }
    }
}

#[cfg(unix)]
impl Drop for DaemonRunner {
    fn drop(&mut self) {
        use duct::unix::HandleExt;
        use libc;

        let _ = self.process.send_signal(libc::SIGTERM);
    }
}
