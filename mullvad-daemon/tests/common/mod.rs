#![allow(dead_code)]

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use duct;
use duct::unix::HandleExt;
use libc;
use os_pipe::{pipe, PipeReader};
use serde::{Deserialize, Serialize};
use talpid_ipc::WsIpcClient;

#[cfg(unix)]
pub fn rpc_file_path() -> PathBuf {
    Path::new("/tmp/.mullvad_rpc_address").to_path_buf()
}

#[cfg(not(unix))]
pub fn rpc_file_path() -> PathBuf {
    ::std::env::temp_dir().join(".mullvad_rpc_address")
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

pub struct DaemonInstance {
    process: Option<duct::Handle>,
    output: Arc<Mutex<BufReader<PipeReader>>>,
    rpc_client: Option<DaemonRpcClient>,
}

impl DaemonInstance {
    pub fn new() -> Self {
        let (reader, writer) = pipe().expect("failed to open pipe to connect to daemon");
        let process = cmd!(
            "../target/debug/mullvad-daemon",
            "--disable-rpc-auth",
            "--resource-dir",
            "dist-assets"
        ).dir("..")
            .stderr_to_stdout()
            .stdout_handle(writer)
            .start()
            .expect("failed to start daemon");

        DaemonInstance {
            process: Some(process),
            output: Arc::new(Mutex::new(BufReader::new(reader))),
            rpc_client: None,
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

    pub fn rpc_client(&mut self) -> &DaemonRpcClient {
        if self.rpc_client.is_none() {
            self.rpc_client = Some(DaemonRpcClient::new().unwrap());
        }

        self.rpc_client.as_ref().unwrap()
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

    fn shutdown(&mut self) -> bool {
        let rpc_client = self.rpc_client
            .take()
            .or_else(|| DaemonRpcClient::new().ok());

        if let Some(rpc_client) = rpc_client {
            rpc_client.shutdown().is_ok()
        } else {
            false
        }
    }

    #[cfg(unix)]
    fn terminate(process: &duct::Handle) -> bool {
        process.send_signal(libc::SIGTERM).is_ok()
    }

    #[cfg(not(unix))]
    fn terminate(process: &duct::Handle) -> bool {
        false
    }
}

impl Drop for DaemonInstance {
    fn drop(&mut self) {
        if let Some(process) = self.process.take() {
            if self.shutdown() || Self::terminate(&process) {
                let process = Arc::new(process);
                let wait_handle = process.clone();

                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(5));
                    match process.try_wait() {
                        Ok(Some(_)) => {}
                        _ => {
                            process.kill().unwrap();
                        }
                    }
                });

                wait_handle.wait();
            } else {
                process.kill().unwrap();
            }
        }
    }
}
