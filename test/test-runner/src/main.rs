use futures::{pin_mut, SinkExt, StreamExt};
use logging::LOGGER;
use std::{
    collections::{BTreeMap, HashMap},
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use tarpc::context;
use tarpc::server::Channel;
use test_rpc::{
    mullvad_daemon::{ServiceStatus, SOCKET_PATH},
    net::SockHandleId,
    package::Package,
    transport::GrpcForwarder,
    AppTrace, Service,
};
use tokio::sync::broadcast::error::TryRecvError;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};

mod app;
mod forward;
mod logging;
mod net;
mod package;
mod sys;

#[derive(Clone)]
pub struct TestServer(pub ());

#[tarpc::server]
impl Service for TestServer {
    async fn install_app(
        self,
        _: context::Context,
        package: Package,
    ) -> Result<(), test_rpc::Error> {
        log::debug!("Installing app");

        package::install_package(package).await?;

        log::debug!("Install complete");

        Ok(())
    }

    async fn uninstall_app(
        self,
        _: context::Context,
        env: HashMap<String, String>,
    ) -> Result<(), test_rpc::Error> {
        log::debug!("Uninstalling app");

        package::uninstall_app(env).await?;

        log::debug!("Uninstalled app");

        Ok(())
    }

    async fn exec(
        self,
        _: context::Context,
        path: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    ) -> Result<test_rpc::ExecResult, test_rpc::Error> {
        log::debug!("Exec {} (args: {args:?})", path);

        let mut cmd = Command::new(&path);
        cmd.args(args);

        // Make sure that PATH is updated
        // TODO: We currently do not need this on non-Windows
        #[cfg(target_os = "windows")]
        cmd.env("PATH", sys::get_system_path_var()?);

        cmd.envs(env);

        let output = cmd.output().await.map_err(|error| {
            log::error!("Failed to exec {}: {error}", path);
            test_rpc::Error::Syscall
        })?;

        let result = test_rpc::ExecResult {
            code: output.status.code(),
            stdout: output.stdout,
            stderr: output.stderr,
        };

        log::debug!("Finished exec: {:?}", result.code);

        Ok(result)
    }

    async fn mullvad_daemon_get_status(
        self,
        _: context::Context,
    ) -> test_rpc::mullvad_daemon::ServiceStatus {
        get_pipe_status()
    }

    async fn find_mullvad_app_traces(
        self,
        _: context::Context,
    ) -> Result<Vec<AppTrace>, test_rpc::Error> {
        app::find_traces()
    }

    async fn get_mullvad_app_cache_dir(
        self,
        _: context::Context,
    ) -> Result<PathBuf, test_rpc::Error> {
        app::find_cache_traces()
    }

    async fn send_tcp(
        self,
        _: context::Context,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), test_rpc::Error> {
        net::send_tcp(interface, bind_addr, destination).await
    }

    async fn send_udp(
        self,
        _: context::Context,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), test_rpc::Error> {
        net::send_udp(interface, bind_addr, destination).await
    }

    async fn send_ping(
        self,
        _: context::Context,
        interface: Option<String>,
        destination: IpAddr,
    ) -> Result<(), test_rpc::Error> {
        net::send_ping(interface.as_deref(), destination).await
    }

    async fn geoip_lookup(
        self,
        _: context::Context,
        mullvad_host: String,
    ) -> Result<test_rpc::AmIMullvad, test_rpc::Error> {
        test_rpc::net::geoip_lookup(mullvad_host).await
    }

    async fn resolve_hostname(
        self,
        _: context::Context,
        hostname: String,
    ) -> Result<Vec<SocketAddr>, test_rpc::Error> {
        Ok(tokio::net::lookup_host(&format!("{hostname}:0"))
            .await
            .map_err(|error| {
                log::debug!("resolve_hostname failed: {error}");
                test_rpc::Error::DnsResolution
            })?
            .collect())
    }

    async fn start_tcp_forward(
        self,
        _: context::Context,
        bind_addr: SocketAddr,
        via_addr: SocketAddr,
    ) -> Result<(SockHandleId, SocketAddr), test_rpc::Error> {
        forward::start_server(bind_addr, via_addr).await
    }

    async fn stop_tcp_forward(
        self,
        _: context::Context,
        id: SockHandleId,
    ) -> Result<(), test_rpc::Error> {
        forward::stop_server(id).await
    }

    async fn get_interface_ip(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<IpAddr, test_rpc::Error> {
        net::get_interface_ip(&interface)
    }

    async fn get_default_interface(self, _: context::Context) -> Result<String, test_rpc::Error> {
        Ok(net::get_default_interface().to_owned())
    }

    async fn poll_output(
        self,
        _: context::Context,
    ) -> Result<Vec<test_rpc::logging::Output>, test_rpc::Error> {
        let mut listener = LOGGER.0.lock().await;
        if let Ok(output) = listener.recv().await {
            let mut buffer = vec![output];
            while let Ok(output) = listener.try_recv() {
                buffer.push(output);
            }
            Ok(buffer)
        } else {
            Err(test_rpc::Error::Logger(
                test_rpc::logging::Error::StandardOutput,
            ))
        }
    }

    async fn try_poll_output(
        self,
        _: context::Context,
    ) -> Result<Vec<test_rpc::logging::Output>, test_rpc::Error> {
        let mut listener = LOGGER.0.lock().await;
        match listener.try_recv() {
            Ok(output) => {
                let mut buffer = vec![output];
                while let Ok(output) = listener.try_recv() {
                    buffer.push(output);
                }
                Ok(buffer)
            }
            Err(TryRecvError::Empty) => Ok(Vec::new()),
            Err(_) => Err(test_rpc::Error::Logger(
                test_rpc::logging::Error::StandardOutput,
            )),
        }
    }

    async fn get_mullvad_app_logs(self, _: context::Context) -> test_rpc::logging::LogOutput {
        logging::get_mullvad_app_logs().await
    }

    async fn restart_mullvad_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        sys::restart_app().await
    }

    /// Stop the Mullvad VPN application.
    async fn stop_mullvad_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        sys::stop_app().await
    }

    /// Start the Mullvad VPN application.
    async fn start_mullvad_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        sys::start_app().await
    }

    async fn set_daemon_log_level(
        self,
        _: context::Context,
        verbosity_level: test_rpc::mullvad_daemon::Verbosity,
    ) -> Result<(), test_rpc::Error> {
        sys::set_daemon_log_level(verbosity_level).await
    }

    async fn set_daemon_environment(
        self,
        _: context::Context,
        env: HashMap<String, String>,
    ) -> Result<(), test_rpc::Error> {
        sys::set_daemon_environment(env).await
    }

    async fn copy_file(
        self,
        _: context::Context,
        src: String,
        dest: String,
    ) -> Result<(), test_rpc::Error> {
        tokio::fs::copy(&src, &dest).await.map_err(|error| {
            log::error!("Failed to copy \"{src}\" to \"{dest}\": {error}");
            test_rpc::Error::Syscall
        })?;
        Ok(())
    }

    /// Write a slice as the entire contents of a file.
    ///
    /// See the documention of [`tokio::fs::write`] for details of the behavior.
    async fn write_file(
        self,
        _: context::Context,
        dest: PathBuf,
        bytes: Vec<u8>,
    ) -> Result<(), test_rpc::Error> {
        tokio::fs::write(&dest, bytes).await.map_err(|error| {
            log::error!(
                "Failed to write to \"{dest}\": {error}",
                dest = dest.display()
            );
            test_rpc::Error::Syscall
        })?;
        Ok(())
    }

    async fn reboot(self, _: context::Context) -> Result<(), test_rpc::Error> {
        sys::reboot()
    }

    async fn make_device_json_old(self, _: context::Context) -> Result<(), test_rpc::Error> {
        app::make_device_json_old().await
    }
}

fn get_pipe_status() -> ServiceStatus {
    match Path::new(SOCKET_PATH).exists() {
        true => ServiceStatus::Running,
        false => ServiceStatus::NotRunning,
    }
}

/// The baud rate of the serial connection between the test manager and the test runner.
/// There is a known issue with setting a baud rate at all or macOS, and the workaround
/// is to set it to zero: https://github.com/serialport/serialport-rs/pull/58
///
/// Keep this constant in sync with `test-manager/src/run_tests.rs`
const BAUD: u32 = if cfg!(target_os = "macos") { 0 } else { 115200 };

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unknown RPC")]
    UnknownRpc,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::init_logger().unwrap();

    let mut args = std::env::args();
    let _ = args.next();
    let path = args.next().expect("serial/COM path must be provided");

    loop {
        log::info!("Connecting to {}", path);

        let serial_stream =
            tokio_serial::SerialStream::open(&tokio_serial::new(&path, BAUD)).unwrap();
        let (runner_transport, mullvad_daemon_transport, _completion_handle) =
            test_rpc::transport::create_server_transports(serial_stream);

        log::info!("Running server");

        tokio::spawn(forward_to_mullvad_daemon_interface(
            mullvad_daemon_transport,
        ));

        let server = tarpc::server::BaseChannel::with_defaults(runner_transport);
        server.execute(TestServer(()).serve()).await;

        log::error!("Restarting server since it stopped");
    }
}

/// Forward data between the test manager and Mullvad management interface socket
async fn forward_to_mullvad_daemon_interface(proxy_transport: GrpcForwarder) {
    const IPC_READ_BUF_SIZE: usize = 16 * 1024;

    let mut srv_read_buf = [0u8; IPC_READ_BUF_SIZE];
    let mut proxy_transport = LengthDelimitedCodec::new().framed(proxy_transport);

    loop {
        // Wait for input from the test manager before connecting to the UDS or named pipe.
        // Connect at the last moment since the daemon may not even be running when the
        // test runner first starts.
        let first_message = match proxy_transport.next().await {
            Some(Ok(bytes)) => {
                if bytes.is_empty() {
                    log::debug!("ignoring EOF from client");
                    continue;
                }
                bytes
            }
            Some(Err(error)) => {
                log::error!("daemon client channel error: {error}");
                break;
            }
            None => break,
        };

        log::info!("mullvad daemon: connecting");

        let mut daemon_socket_endpoint =
            match parity_tokio_ipc::Endpoint::connect(SOCKET_PATH).await {
                Ok(uds_endpoint) => uds_endpoint,
                Err(error) => {
                    log::error!("mullvad daemon: failed to connect: {error}");
                    // send EOF
                    let _ = proxy_transport.send(bytes::Bytes::new()).await;
                    continue;
                }
            };

        log::info!("mullvad daemon: connected");

        if let Err(error) = daemon_socket_endpoint.write_all(&first_message).await {
            log::error!("writing to uds failed: {error}");
            continue;
        }

        loop {
            let srv_read = daemon_socket_endpoint.read(&mut srv_read_buf);
            pin_mut!(srv_read);

            match futures::future::select(srv_read, proxy_transport.next()).await {
                futures::future::Either::Left((read, _)) => match read {
                    Ok(num_bytes) => {
                        if num_bytes == 0 {
                            log::debug!("uds EOF; restarting server");
                            break;
                        }
                        if let Err(error) = proxy_transport
                            .send(srv_read_buf[..num_bytes].to_vec().into())
                            .await
                        {
                            log::error!("writing to client channel failed: {error}");
                            break;
                        }
                    }
                    Err(error) => {
                        log::error!("reading from uds failed: {error}");
                        let _ = proxy_transport.send(bytes::Bytes::new()).await;
                        break;
                    }
                },
                futures::future::Either::Right((read, _)) => match read {
                    Some(Ok(bytes)) => {
                        if bytes.is_empty() {
                            log::debug!("management interface EOF; restarting server");
                            break;
                        }
                        if let Err(error) = daemon_socket_endpoint.write_all(&bytes).await {
                            log::error!("writing to uds failed: {error}");
                            break;
                        }
                    }
                    Some(Err(error)) => {
                        log::error!("daemon client channel error: {error}");
                        break;
                    }
                    None => break,
                },
            }
        }

        log::info!("mullvad daemon: disconnected");
    }
}
