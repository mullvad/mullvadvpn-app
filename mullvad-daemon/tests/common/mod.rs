#![allow(dead_code)]

#[cfg(unix)]
extern crate libc;

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use duct;
use os_pipe::{pipe, PipeReader};
use serde::{Deserialize, Serialize};
use talpid_ipc::WsIpcClient;

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
        let windows_directory = ::std::env::var_os("WINDIR").unwrap();
        PathBuf::from(windows_directory)
            .join("Temp")
            .join(".mullvad_rpc_address")
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

pub struct DaemonRpcClient {
    address: String,
}

impl DaemonRpcClient {
    fn new() -> Result<Self, String> {
        let rpc_file = File::open(rpc_file_path())
            .map_err(|error| format!("failed to open RPC address file: {}", error))?;
        let reader = BufReader::new(rpc_file);
        let mut lines = reader.lines();
        let address = lines
            .next()
            .ok_or("RPC address file is empty".to_string())?
            .map_err(|error| format!("failed to read address from RPC address file: {}", error))?;

        Ok(DaemonRpcClient { address })
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.call("shutdown", &[] as &[u8; 0])
    }

    pub fn call<A, O>(&self, method: &str, args: &A) -> Result<O, String>
    where
        A: Serialize,
        O: for<'de> Deserialize<'de>,
    {
        let mut rpc_client = WsIpcClient::new(self.address.clone())
            .map_err(|error| format!("unable to create RPC client: {}", error))?;

        rpc_client
            .call(method, args)
            .map_err(|error| format!("RPC request failed: {}", error))
    }
}

pub struct DaemonRunner {
    process: Option<duct::Handle>,
    output: Arc<Mutex<BufReader<PipeReader>>>,
}

impl DaemonRunner {
    pub fn spawn() -> Self {
        prepare_relay_list("../dist-assets/relays.json");

        let (reader, writer) = pipe().expect("failed to open pipe to connect to daemon");
        let process = cmd!(
            DAEMON_EXECUTABLE_PATH,
            "-v",
            "--disable-rpc-auth",
            "--resource-dir",
            "dist-assets"
        ).dir("..")
            .stderr_to_stdout()
            .stdout_handle(writer)
            .start()
            .expect("failed to start daemon");

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

    #[cfg(unix)]
    fn request_clean_shutdown(&mut self, process: &mut duct::Handle) -> bool {
        use duct::unix::HandleExt;

        process.send_signal(libc::SIGTERM).is_ok()
    }

    #[cfg(not(unix))]
    fn request_clean_shutdown(&mut self, _: &mut duct::Handle) -> bool {
        if let Ok(rpc_client) = DaemonRpcClient::new() {
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
