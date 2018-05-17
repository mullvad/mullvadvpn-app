#[macro_use]
extern crate duct;
#[cfg(unix)]
extern crate libc;
extern crate mullvad_ipc_client;
extern crate notify;
extern crate os_pipe;
extern crate tempdir;

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use mullvad_ipc_client::{rpc_file_path, DaemonRpcClient};
use notify::{op, RawEvent, RecursiveMode, Watcher};
use os_pipe::{pipe, PipeReader};
use tempdir::TempDir;

use self::platform_specific::*;

pub const MOCK_OPENVPN_COMMAND_LINE_FILE: &str = "mock_openvpn_command_line";

#[cfg(unix)]
mod platform_specific {
    pub static DAEMON_EXECUTABLE_PATH: &str = "../target/debug/mullvad-daemon";
    pub static MOCK_OPENVPN_EXECUTABLE_PATH: &str = "../target/debug/mock_openvpn";
    pub static OPENVPN_EXECUTABLE_FILE: &str = "openvpn";
    #[cfg(target_os = "linux")]
    pub static TALPID_OPENVPN_PLUGIN_FILE: &str = "libtalpid_openvpn_plugin.so";
    #[cfg(target_os = "macos")]
    pub static TALPID_OPENVPN_PLUGIN_FILE: &str = "libtalpid_openvpn_plugin.dylib";
}

#[cfg(not(unix))]
mod platform_specific {
    pub static DAEMON_EXECUTABLE_PATH: &str = r"..\target\debug\mullvad-daemon.exe";
    pub static MOCK_OPENVPN_EXECUTABLE_PATH: &str = "../target/debug/mock_openvpn.exe";
    pub static OPENVPN_EXECUTABLE_FILE: &str = "openvpn.exe";
    pub static TALPID_OPENVPN_PLUGIN_FILE: &str = "talpid_openvpn_plugin.dll";
}

pub fn wait_for_file<P: AsRef<Path>>(file_path: P, timeout: Duration) {
    let file_path = file_path.as_ref();
    let file_name = file_path.file_name();
    let parent_dir = file_path.parent().expect("missing file parent directory");

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::raw_watcher(tx).expect("failed to listen for file system events");
    let start = Instant::now();
    let mut remaining_time = Some(timeout);

    watcher
        .watch(parent_dir, RecursiveMode::NonRecursive)
        .expect("failed to listen for file system events on directory");

    if !file_path.exists() {
        while let Some(wait_time) = remaining_time {
            let event = rx.recv_timeout(wait_time);

            if let Ok(RawEvent {
                path: Some(path),
                op: Ok(op),
                ..
            }) = event
            {
                if op.contains(op::CLOSE_WRITE) && path.file_name() == file_name {
                    break;
                }
            }

            remaining_time = timeout.checked_sub(start.elapsed());
        }
    }
}

fn prepare_fake_resource_dir() -> (TempDir, PathBuf) {
    let temp_dir =
        TempDir::new("mullvad-daemon-test").expect("failed to create temporary directory");
    let resource_dir = temp_dir.path().join("resource-dir");
    let relay_list = resource_dir.join("relays.json");
    let openvpn_binary = resource_dir.join(OPENVPN_EXECUTABLE_FILE);
    let talpid_openvpn_plugin = resource_dir.join(TALPID_OPENVPN_PLUGIN_FILE);

    fs::create_dir(&resource_dir).expect("failed to resource directory");
    fs::copy(MOCK_OPENVPN_EXECUTABLE_PATH, openvpn_binary)
        .expect("failed to copy mock OpenVPN binary");
    File::create(talpid_openvpn_plugin).expect("failed to create mock Talpid OpenVPN plugin");

    prepare_relay_list(relay_list);

    (temp_dir, resource_dir)
}

fn prepare_relay_list<T: AsRef<Path>>(path: T) {
    let path = path.as_ref();

    if !path.exists() {
        let mut relays = File::create(path).expect("failed to create relay list file");

        writeln!(
            relays,
            "{}",
            r#"{
                "countries": [{
                    "name": "Mockland",
                    "code": "fake",
                    "latitude": -91,
                    "longitude": 0,
                    "has_active_relays": false,
                    "relays": [{
                        "hostname": "fake-mockland",
                        "ipv4_addr_in": "192.168.0.100",
                        "ipv4_addr_exit": "192.168.0.101",
                        "include_in_country": true,
                        "weight": 100,
                        "tunnels": {
                            "openvpn": [ { "port": 10000, "protocol": "udp" } ],
                            "wireguard": [],
                        },
                    }],
                }]
            }"#
        ).expect("failed to write relay list");
    }
}

pub struct DaemonRunner {
    process: Option<duct::Handle>,
    output: Arc<Mutex<BufReader<PipeReader>>>,
    mock_openvpn_command_line_file: PathBuf,
    _temp_dir: TempDir,
}

impl DaemonRunner {
    pub fn spawn() -> Self {
        let (temp_dir, resource_dir) = prepare_fake_resource_dir();

        let (reader, writer) = pipe().expect("failed to open pipe to connect to daemon");
        let process = cmd!(
            DAEMON_EXECUTABLE_PATH,
            "-v",
            "--resource-dir",
            &resource_dir.display().to_string(),
            "--cache-dir",
            "./"
        ).dir("..")
            .stderr_to_stdout()
            .stdout_handle(writer)
            .start()
            .expect("failed to start daemon");

        let mock_openvpn_command_line_file = temp_dir.path().join(MOCK_OPENVPN_COMMAND_LINE_FILE);

        DaemonRunner {
            process: Some(process),
            output: Arc::new(Mutex::new(BufReader::new(reader))),
            mock_openvpn_command_line_file,
            _temp_dir: temp_dir,
        }
    }

    pub fn mock_openvpn_command_line_file(&self) -> &Path {
        &self.mock_openvpn_command_line_file
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

    pub fn rpc_client(&mut self) -> Result<DaemonRpcClient, String> {
        let rpc_file = rpc_file_path()
            .map_err(|error| format!("Failed to build RPC connection file path: {}", error))?;

        if !rpc_file.exists() {
            wait_for_file(rpc_file, Duration::from_secs(10));
        }

        DaemonRpcClient::without_rpc_file_security_check()
            .map_err(|error| format!("Failed to create RPC client: {}", error))
    }

    #[cfg(unix)]
    fn request_clean_shutdown(&mut self, process: &mut duct::Handle) -> bool {
        use duct::unix::HandleExt;

        process.send_signal(libc::SIGTERM).is_ok()
    }

    #[cfg(not(unix))]
    fn request_clean_shutdown(&mut self, _: &mut duct::Handle) -> bool {
        if let Ok(mut rpc_client) = self.rpc_client() {
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

        if let Ok(file_path) = rpc_file_path() {
            let _ = fs::remove_file(file_path);
        }
    }
}
