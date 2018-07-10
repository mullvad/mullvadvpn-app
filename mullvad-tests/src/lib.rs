#[macro_use]
extern crate duct;
#[cfg(unix)]
extern crate libc;
extern crate mullvad_ipc_client;
extern crate mullvad_paths;
extern crate notify;
extern crate openvpn_plugin;
extern crate talpid_ipc;
extern crate tempfile;

pub mod mock_openvpn;

use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use std::{cmp, thread};

use mullvad_ipc_client::DaemonRpcClient;
use mullvad_paths::resources::API_CA_FILENAME;
use notify::{RawEvent, RecommendedWatcher, RecursiveMode, Watcher};
use openvpn_plugin::types::OpenVpnPluginEvent;
use talpid_ipc::WsIpcClient;
use tempfile::TempDir;

use self::mock_openvpn::MOCK_OPENVPN_ARGS_FILE;
use self::platform_specific::*;

pub use self::notify::op::{self as watch_event, Op as WatchEvent};

type Result<T> = ::std::result::Result<T, String>;

pub const ASSETS_DIR: &str = "../dist-assets";

#[cfg(unix)]
mod platform_specific {
    pub const DAEMON_EXECUTABLE_PATH: &str = "../target/debug/mullvad-daemon";
    pub const MOCK_OPENVPN_EXECUTABLE_PATH: &str = "../target/debug/mock_openvpn";
    pub const OPENVPN_EXECUTABLE_FILE: &str = "openvpn";
    #[cfg(target_os = "linux")]
    pub const TALPID_OPENVPN_PLUGIN_FILE: &str = "libtalpid_openvpn_plugin.so";
    #[cfg(target_os = "macos")]
    pub const TALPID_OPENVPN_PLUGIN_FILE: &str = "libtalpid_openvpn_plugin.dylib";
}

#[cfg(not(unix))]
mod platform_specific {
    pub const DAEMON_EXECUTABLE_PATH: &str = r"..\target\debug\mullvad-daemon.exe";
    pub const MOCK_OPENVPN_EXECUTABLE_PATH: &str = "../target/debug/mock_openvpn.exe";
    pub const OPENVPN_EXECUTABLE_FILE: &str = "openvpn.exe";
    pub const TALPID_OPENVPN_PLUGIN_FILE: &str = "talpid_openvpn_plugin.dll";
}

pub struct PathWatcher {
    events: mpsc::Receiver<RawEvent>,
    path: PathBuf,
    timeout: Duration,
    _watcher: RecommendedWatcher,
}

impl PathWatcher {
    pub fn watch<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file_path = file_path.as_ref();
        let parent_dir = file_path
            .parent()
            .ok_or_else(|| "Missing file parent directory")?;

        let absolute_parent_dir = parent_dir
            .canonicalize()
            .map_err(|_| "Failed to get absolute path to watch")?;
        let file_name = file_path
            .file_name()
            .ok_or_else(|| "Missing file name of file path to watch")?;
        let absolute_file_path = absolute_parent_dir.join(file_name);

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::raw_watcher(tx).map_err(|_| {
            format!(
                "Failed to create watcher of file system events to watch {}",
                file_path.display()
            )
        })?;

        watcher
            .watch(absolute_parent_dir, RecursiveMode::Recursive)
            .map_err(|_| {
                format!(
                    "Failed to start watching for file system events from {}",
                    file_path.display()
                )
            })?;

        Ok(PathWatcher {
            events: rx,
            path: absolute_file_path,
            timeout: Duration::from_secs(5),
            _watcher: watcher,
        })
    }

    pub fn set_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn assert_create_write_close_sequence(&mut self) {
        assert_eq!(self.next(), Some(watch_event::CREATE));
        assert_eq!(self.next(), Some(watch_event::WRITE));

        #[cfg(not(target_os = "linux"))]
        self.wait_for_burst_of_events(Duration::from_secs(1));

        #[cfg(target_os = "linux")]
        loop {
            match self.next() {
                Some(watch_event::WRITE) => continue,
                event => {
                    assert_eq!(event, Some(watch_event::CLOSE_WRITE));
                    break;
                }
            }
        }
    }

    /// Waits for a burst of file events.
    ///
    /// Here, a burst of events is defined as a series of events that are emitted with less than one
    /// second between each of them.
    ///
    /// The `max_wait_time` defines the maximum time to wait for all of the events. If a burst of
    /// events is emitted that is longer than the specified time, the function will return before
    /// all events have been received.
    pub fn wait_for_burst_of_events(&mut self, max_wait_time: Duration) {
        const EVENT_INTERVAL: Duration = Duration::from_secs(1);

        let start = Instant::now();
        let original_timeout = self.timeout;

        // We wait at most for the maximum waiting time for the first event to arrive
        self.timeout = max_wait_time;

        if self.next().is_some() {
            while let Some(remaining_time) = max_wait_time.checked_sub(start.elapsed()) {
                // Avoid exceeding the maximum wait time
                self.timeout = cmp::min(EVENT_INTERVAL, remaining_time);

                if self.next().is_none() {
                    break;
                }
            }
        }

        self.timeout = original_timeout;
    }
}

impl Iterator for PathWatcher {
    type Item = WatchEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();

        while let Some(remaining_time) = self.timeout.checked_sub(start.elapsed()) {
            match self.events.recv_timeout(remaining_time) {
                Ok(RawEvent {
                    path: Some(path),
                    op: Ok(op),
                    ..
                }) => if path == self.path {
                    return Some(op);
                } else {
                    continue;
                },
                Ok(_) => continue,
                Err(_) => return None,
            }
        }

        None
    }
}

pub fn wait_for_file<P: AsRef<Path>>(file_path: P) {
    let file_path = file_path.as_ref();
    let mut watcher = PathWatcher::watch(&file_path).expect(&format!(
        "Failed to watch file for changes: {}",
        file_path.display()
    ));

    if !file_path.exists() {
        // No event has been emitted yet. Wait for the initial create event.
        assert_eq!(watcher.next(), Some(watch_event::CREATE));
    }

    // The file was created, so at least one event was emitted. Assume the write burst has started
    // and wait for a short amount of time until it completes.
    watcher.wait_for_burst_of_events(Duration::from_secs(1));
}

fn prepare_test_dirs() -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temporary daemon directory");
    let cache_dir = temp_dir.path().join("cache");
    let resource_dir = temp_dir.path().join("resource-dir");
    let settings_dir = temp_dir.path().join("settings");

    fs::create_dir(&cache_dir).expect("Failed to create cache directory");
    fs::create_dir(&resource_dir).expect("Failed to create resource directory");
    fs::create_dir(&settings_dir).expect("Failed to create settings directory");

    prepare_resource_dir(&resource_dir);

    (temp_dir, cache_dir, resource_dir, settings_dir)
}

fn prepare_resource_dir(resource_dir: &Path) {
    let assets_dir = PathBuf::from(ASSETS_DIR);
    let api_certificate_src = assets_dir.join(API_CA_FILENAME);
    let openvpn_binary = resource_dir.join(OPENVPN_EXECUTABLE_FILE);
    let talpid_openvpn_plugin = resource_dir.join(TALPID_OPENVPN_PLUGIN_FILE);
    let api_certificate_dst = resource_dir.join(API_CA_FILENAME);

    fs::copy(api_certificate_src, api_certificate_dst).expect("Failed to copy API certificate");
    fs::copy(MOCK_OPENVPN_EXECUTABLE_PATH, openvpn_binary)
        .expect("Failed to copy mock OpenVPN binary");
    File::create(talpid_openvpn_plugin).expect("Failed to create mock Talpid OpenVPN plugin");

    prepare_relay_list(resource_dir.join("relays.json"));
}

fn prepare_relay_list<T: AsRef<Path>>(path: T) {
    fs::write(
        path,
        r#"{
            "countries": [{
                "name": "Sweden",
                "code": "se",
                "cities": [{
                    "name": "Gothenburg",
                    "code": "got",
                    "latitude": 57.70887,
                    "longitude": 11.97456,
                    "relays": [{
                        "hostname": "fakehost",
                        "ipv4_addr_in": "192.168.0.100",
                        "ipv4_addr_exit": "192.168.0.101",
                        "include_in_country": true,
                        "weight": 100,
                        "tunnels": {
                            "openvpn": [ { "port": 1000, "protocol": "udp" } ],
                            "wireguard": []
                        }
                    }]
                }]
            }]
        }"#,
    ).expect("Failed to create mock relay list file");
}

pub struct DaemonRunner {
    process: Option<duct::Handle>,
    mock_openvpn_args_file: PathBuf,
    rpc_address_file: PathBuf,
    _temp_dir: TempDir,
}

impl DaemonRunner {
    pub fn spawn_with_real_rpc_address_file() -> Self {
        Self::spawn_internal(false)
    }

    pub fn spawn() -> Self {
        Self::spawn_internal(true)
    }

    fn spawn_internal(mock_rpc_address_file: bool) -> Self {
        let (temp_dir, cache_dir, resource_dir, settings_dir) = prepare_test_dirs();
        let mock_openvpn_args_file = temp_dir.path().join(MOCK_OPENVPN_ARGS_FILE);
        let rpc_address_file = if mock_rpc_address_file {
            temp_dir.path().join(".mullvad_rpc_address")
        } else {
            mullvad_paths::get_rpc_address_path().expect("Failed to build RPC connection file path")
        };

        let mut expression = cmd!(DAEMON_EXECUTABLE_PATH, "-v", "--disable-log-to-file")
            .dir("..")
            .env("MULLVAD_CACHE_DIR", cache_dir)
            .env("MULLVAD_RESOURCE_DIR", resource_dir)
            .env("MULLVAD_SETTINGS_DIR", settings_dir)
            .env("MOCK_OPENVPN_ARGS_FILE", mock_openvpn_args_file.clone())
            .stdout_null()
            .stderr_null();

        if mock_rpc_address_file {
            expression = expression.env(
                "MULLVAD_RPC_ADDRESS_PATH",
                rpc_address_file.display().to_string(),
            );
        }

        let process = expression.start().expect("Failed to start daemon");

        DaemonRunner {
            process: Some(process),
            mock_openvpn_args_file,
            rpc_address_file,
            _temp_dir: temp_dir,
        }
    }

    pub fn mock_openvpn_args_file(&self) -> &Path {
        &self.mock_openvpn_args_file
    }

    pub fn rpc_client(&mut self) -> Result<DaemonRpcClient> {
        wait_for_file(&self.rpc_address_file);

        DaemonRpcClient::with_insecure_rpc_address_file(&self.rpc_address_file)
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

        let _ = fs::remove_file(&self.rpc_address_file);
    }
}

pub struct MockOpenVpnPluginRpcClient {
    credentials: String,
    rpc: WsIpcClient,
}

impl MockOpenVpnPluginRpcClient {
    pub fn new(address: String, credentials: String) -> Result<Self> {
        let rpc = WsIpcClient::connect(&address).map_err(|error| {
            format!("Failed to create Mock OpenVPN plugin RPC client: {}", error)
        })?;

        Ok(MockOpenVpnPluginRpcClient { rpc, credentials })
    }

    pub fn authenticate(&mut self) -> Result<bool> {
        self.rpc
            .call("authenticate", &[&self.credentials])
            .map_err(|error| format!("Failed to authenticate mock OpenVPN IPC client: {}", error))
    }

    pub fn authenticate_with(&mut self, credentials: &str) -> Result<bool> {
        self.rpc
            .call("authenticate", &[credentials])
            .map_err(|error| format!("Failed to authenticate mock OpenVPN IPC client: {}", error))
    }

    pub fn up(&mut self) -> Result<()> {
        let mut env: HashMap<String, String> = HashMap::new();

        env.insert("dev".to_owned(), "lo".to_owned());
        env.insert("ifconfig_local".to_owned(), "10.0.0.10".to_owned());
        env.insert("route_vpn_gateway".to_owned(), "10.0.0.1".to_owned());

        self.send_event(OpenVpnPluginEvent::Up, env)
    }

    pub fn route_predown(&mut self) -> Result<()> {
        self.send_event(OpenVpnPluginEvent::RoutePredown, HashMap::new())
    }

    fn send_event(
        &mut self,
        event: OpenVpnPluginEvent,
        env: HashMap<String, String>,
    ) -> Result<()> {
        self.rpc
            .call("openvpn_event", &(event, env))
            .map_err(|error| format!("Failed to send mock OpenVPN event {:?}: {}", event, error))
    }
}
