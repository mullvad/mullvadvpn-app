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

#[derive(err_derive::Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    #[error(display = "Test runner RPC failed")]
    Tarpc(#[error(source)] tarpc::client::RpcError),
    #[error(display = "Syscall failed")]
    Syscall,
    #[error(display = "Interface not found")]
    InterfaceNotFound,
    #[error(display = "HTTP request failed")]
    HttpRequest(String),
    #[error(display = "Failed to deserialize HTTP body")]
    DeserializeBody,
    #[error(display = "DNS resolution failed")]
    DnsResolution,
    #[error(display = "Test runner RPC timed out")]
    TestRunnerTimeout,
    #[error(display = "Package error")]
    Package(#[error(source)] package::Error),
    #[error(display = "Logger error")]
    Logger(#[error(source)] logging::Error),
    #[error(display = "Failed to send UDP datagram")]
    SendUdp,
    #[error(display = "Failed to send TCP segment")]
    SendTcp,
    #[error(display = "Failed to send ping")]
    Ping,
    #[error(display = "Failed to get or set registry value")]
    Registry(String),
    #[error(display = "Failed to change the service")]
    Service(String),
    #[error(display = "Could not read from or write to the file system")]
    FileSystem(String),
    #[error(display = "Could not serialize or deserialize file")]
    FileSerialization(String),
    #[error(display = "User must be logged in but is not")]
    UserNotLoggedIn(String),
    #[error(display = "Invalid URL")]
    InvalidUrl,
    #[error(display = "Timeout")]
    Timeout,
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

        /// Return the OS of the guest.
        async fn get_os() -> meta::Os;

        /// Return status of the system service.
        async fn mullvad_daemon_get_status() -> mullvad_daemon::ServiceStatus;

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
        async fn send_ping(interface: Option<String>, destination: IpAddr) -> Result<(), Error>;

        /// Fetch the current location.
        async fn geoip_lookup(mullvad_host: String) -> Result<AmIMullvad, Error>;

        /// Returns the IP of the given interface.
        async fn get_interface_ip(interface: String) -> Result<IpAddr, Error>;

        /// Returns the name of the default interface.
        async fn get_default_interface() -> Result<String, Error>;

        /// Perform DNS resolution.
        async fn resolve_hostname(hostname: String) -> Result<Vec<SocketAddr>, Error>;

        /// Restart the Mullvad VPN application.
        async fn restart_app() -> Result<(), Error>;

        /// Stop the Mullvad VPN application.
        async fn stop_app() -> Result<(), Error>;

        /// Start the Mullvad VPN application.
        async fn start_app() -> Result<(), Error>;

        /// Sets the log level of the daemon service, the verbosity level represents the number of
        /// `-v`s passed on the command line. This will restart the daemon system service.
        async fn set_daemon_log_level(
            verbosity_level: mullvad_daemon::Verbosity,
        ) -> Result<(), Error>;

        /// Set environment variables for the daemon service. This will restart the daemon system service.
        async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), Error>;

        /// Copy a file from `src` to `dest` on the test runner.
        async fn copy_file(src: String, dest: String) -> Result<(), Error>;

        /// Write arbitrary bytes to some file `dest` on the test runner.
        async fn write_file(dest: PathBuf, bytes: Vec<u8>) -> Result<(), Error>;

        async fn reboot() -> Result<(), Error>;

        async fn set_mullvad_daemon_service_state(on: bool) -> Result<(), Error>;

        async fn make_device_json_old() -> Result<(), Error>;
    }
}

pub use client::ServiceClient;
pub use service::{Service, ServiceRequest, ServiceResponse};
