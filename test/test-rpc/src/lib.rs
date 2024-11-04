use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

pub mod client;
pub mod logging;
pub mod meta;
pub mod mullvad_daemon;
pub mod net;
pub mod package;
pub mod transport;

/// Unprivileged user. This is used for things like spawning processes.
/// This is also used as the password for the same user, as is common practice.
pub const UNPRIVILEGED_USER: &str = "mole";

#[derive(thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    #[error("Test runner RPC failed")]
    Tarpc(#[from] tarpc::client::RpcError),
    #[error("Syscall failed")]
    Syscall,
    #[error("Internal IO error occurred: {0}")]
    Io(String),
    #[error("Interface not found")]
    InterfaceNotFound,
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),
    #[error("Failed to deserialize HTTP body")]
    DeserializeBody,
    #[error("DNS resolution failed")]
    DnsResolution,
    #[error("Test runner RPC timed out")]
    TestRunnerTimeout,
    #[error("Package error")]
    Package(#[from] package::Error),
    #[error("Logger error")]
    Logger(#[from] logging::Error),
    #[error("Failed to send UDP datagram")]
    SendUdp,
    #[error("Failed to send TCP segment")]
    SendTcp,
    #[error("Failed to send ping: {0}")]
    Ping(String),
    #[error("Failed to get or set registry value: {0}")]
    Registry(String),
    #[error("Failed to change the service: {0}")]
    Service(String),
    #[error("Could not read from or write to the file system: {0}")]
    FileSystem(String),
    #[error("Could not serialize or deserialize file: {0}")]
    FileSerialization(String),
    #[error("User must be logged in but is not: {0}")]
    UserNotLoggedIn(String),
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Timeout")]
    Timeout,
    #[error("TCP forward error")]
    TcpForward,
    #[error("Unknown process ID: {0}")]
    UnknownPid(u32),
    #[error("Failed to join tokio task: {0}")]
    TokioJoinError(String),
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Convenient mapping from a Tokio error to the test_rpc Error type.
    pub fn from_tokio_join_error(error: tokio::task::JoinError) -> Error {
        Error::TokioJoinError(error.to_string())
    }
}

/// Response from am.i.mullvad.net
#[derive(Debug, Serialize, Deserialize)]
pub struct AmIMullvad {
    pub ip: IpAddr,
    pub mullvad_exit_ip: bool,
    pub mullvad_exit_ip_hostname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecResult {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl ExecResult {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnOpts {
    pub path: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
}

impl SpawnOpts {
    pub fn new(path: impl Into<String>) -> SpawnOpts {
        SpawnOpts {
            path: path.into(),
            args: Default::default(),
            env: Default::default(),
            attach_stdin: Default::default(),
            attach_stdout: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AppTrace {
    Path(PathBuf),
}

mod service {
    use std::collections::HashMap;

    pub use super::*;

    #[tarpc::service]
    pub trait Service {
        /// Install app package.
        async fn install_app(package_path: package::Package) -> Result<(), Error>;

        /// Remove app package.
        async fn uninstall_app(env: HashMap<String, String>) -> Result<(), Error>;

        /// Execute a program.
        async fn exec(
            path: String,
            args: Vec<String>,
            env: BTreeMap<String, String>,
        ) -> Result<ExecResult, Error>;

        /// Get the output of the runners stdout logs since the last time this function was called.
        /// Block if there is no output until some output is provided by the runner.
        async fn poll_output() -> Result<Vec<logging::Output>, Error>;

        /// Get the output of the runners stdout logs since the last time this function was called.
        /// Block if there is no output until some output is provided by the runner.
        async fn try_poll_output() -> Result<Vec<logging::Output>, Error>;

        async fn get_mullvad_app_logs() -> logging::LogOutput;

        /// Return status of the system service.
        async fn mullvad_daemon_get_status() -> mullvad_daemon::ServiceStatus;

        /// Return version number of installed daemon.
        async fn mullvad_version() -> Result<String, Error>;

        /// Returns all Mullvad app files, directories, and other data found on the system.
        async fn find_mullvad_app_traces() -> Result<Vec<AppTrace>, Error>;

        async fn get_mullvad_app_cache_dir() -> Result<PathBuf, Error>;

        /// Send TCP packet
        async fn send_tcp(
            interface: Option<String>,
            bind_addr: SocketAddr,
            destination: SocketAddr,
        ) -> Result<(), Error>;

        /// Send UDP packet
        async fn send_udp(
            interface: Option<String>,
            bind_addr: SocketAddr,
            destination: SocketAddr,
        ) -> Result<(), Error>;

        /// Send ICMP
        async fn send_ping(
            destination: IpAddr,
            interface: Option<String>,
            size: usize,
        ) -> Result<(), Error>;

        /// Fetch the current location.
        async fn geoip_lookup(mullvad_host: String) -> Result<AmIMullvad, Error>;

        /// Returns the IP of the given interface.
        async fn get_interface_ip(interface: String) -> Result<IpAddr, Error>;

        /// Returns the MTU of the given interface.
        async fn get_interface_mtu(interface: String) -> Result<u16, Error>;

        /// Returns the MAC address of the given interface.
        async fn get_interface_mac(interface: String) -> Result<Option<[u8; 6]>, Error>;

        /// Returns the name of the default interface.
        async fn get_default_interface() -> Result<String, Error>;

        /// Perform DNS resolution.
        async fn resolve_hostname(hostname: String) -> Result<Vec<SocketAddr>, Error>;

        /// Start forwarding TCP bound to the given address. Return an ID that can be used with
        /// `stop_tcp_forward`, and the address that the listening socket was actually bound to.
        async fn start_tcp_forward(
            bind_addr: SocketAddr,
            via_addr: SocketAddr,
        ) -> Result<(net::SockHandleId, SocketAddr), Error>;

        /// Stop forwarding TCP that was previously started with `start_tcp_forward`.
        async fn stop_tcp_forward(id: net::SockHandleId) -> Result<(), Error>;

        /// Restart the Mullvad VPN application.
        async fn restart_mullvad_daemon() -> Result<(), Error>;

        /// Stop the Mullvad VPN application.
        async fn stop_mullvad_daemon() -> Result<(), Error>;

        /// Start the Mullvad VPN application.
        async fn start_mullvad_daemon() -> Result<(), Error>;

        /// Sets the log level of the daemon service, the verbosity level represents the number of
        /// `-v`s passed on the command line. This will restart the daemon system service.
        async fn set_daemon_log_level(
            verbosity_level: mullvad_daemon::Verbosity,
        ) -> Result<(), Error>;

        /// Set environment variables for the daemon service. This will restart the daemon system
        /// service.
        async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), Error>;

        /// Get the environment variables for the running daemon service.
        async fn get_daemon_environment() -> Result<HashMap<String, String>, Error>;

        /// Copy a file from `src` to `dest` on the test runner.
        async fn copy_file(src: String, dest: String) -> Result<(), Error>;

        /// Write arbitrary bytes to some file `dest` on the test runner.
        async fn write_file(dest: PathBuf, bytes: Vec<u8>) -> Result<(), Error>;

        async fn reboot() -> Result<(), Error>;

        async fn make_device_json_old() -> Result<(), Error>;

        /// Spawn a child process and return the PID.
        async fn spawn(opts: SpawnOpts) -> Result<u32, Error>;

        /// Read from stdout of a process spawned through [Service::spawn].
        ///
        /// Process must have been spawned with `attach_stdout`.
        /// Returns `None` if process stdout is closed.
        async fn read_child_stdout(pid: u32) -> Result<Option<String>, Error>;

        /// Write to stdin of a process spawned through [Service::spawn].
        ///
        /// Process must have been spawned with `attach_stdin`.
        async fn write_child_stdin(pid: u32, data: String) -> Result<(), Error>;

        /// Close stdin of a process spawned through [Service::spawn].
        ///
        /// Process must have been spawned with `attach_stdin`.
        async fn close_child_stdin(pid: u32) -> Result<(), Error>;

        /// Kill a process spawned through [Service::spawn].
        async fn kill_child(pid: u32) -> Result<(), Error>;

        /// Returns operating system details
        async fn get_os_version() -> Result<meta::OsVersion, Error>;
    }
}

pub use client::ServiceClient;
pub use service::{Service, ServiceRequest, ServiceResponse};
