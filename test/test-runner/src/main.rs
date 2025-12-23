use futures::{FutureExt, SinkExt, StreamExt, pin_mut, select, select_biased};
use logging::LOGGER;
use std::{
    collections::{BTreeMap, HashMap},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::{Duration, SystemTime},
};
use util::OnDrop;

use tarpc::{context, server::Channel};
use test_rpc::{
    AppTrace, Service, SpawnOpts, UNPRIVILEGED_USER, meta::OsVersion, mullvad_daemon::SOCKET_PATH,
    net::SockHandleId, package::Package, transport::GrpcForwarder,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::{Mutex, broadcast::error::TryRecvError, oneshot},
    task,
    time::sleep,
};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};

mod app;
mod forward;
mod logging;
mod net;
mod package;
mod sys;
mod util;

#[derive(Clone, Default)]
pub struct TestServer(Arc<Mutex<State>>);

#[derive(Default)]
struct State {
    spawned_procs: HashMap<u32, SpawnedProcess>,
}

struct SpawnedProcess {
    stdout: Option<ChildStdout>,
    stdin: Option<ChildStdin>,

    #[expect(dead_code)]
    abort_handle: OnDrop,
}

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
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());
        cmd.args(args);

        #[cfg(target_os = "windows")]
        {
            // Make sure that PATH is updated
            cmd.env("PATH", sys::get_system_path_var()?);
            if let Some(home_dir) = dirs::home_dir() {
                cmd.env("USERPROFILE", home_dir);
            }
        }

        #[cfg(unix)]
        if let Some(home_dir) = dirs::home_dir() {
            cmd.env("HOME", home_dir);
        }

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
        sys::get_daemon_status()
    }

    /// Get the installed app version
    async fn mullvad_version(self, _: context::Context) -> Result<String, test_rpc::Error> {
        app::version().await
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
        destination: IpAddr,
        interface: Option<String>,
        size: usize,
    ) -> Result<(), test_rpc::Error> {
        net::send_ping(destination, interface.as_deref(), size)
            .await
            .map_err(|e| test_rpc::Error::Ping(e.to_string()))
    }

    async fn geoip_lookup(
        self,
        ctx: context::Context,
        mullvad_host: String,
    ) -> Result<test_rpc::AmIMullvad, test_rpc::Error> {
        let timeout = ctx
            .deadline
            .duration_since(SystemTime::now())
            .ok()
            // account for some time to send the RPC response
            .and_then(|d| d.checked_sub(Duration::from_millis(500)))
            .unwrap_or_default();

        test_rpc::net::geoip_lookup(mullvad_host, timeout).await
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
        forward::stop_server(id)
    }

    async fn get_interface_ip(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<IpAddr, test_rpc::Error> {
        net::get_interface_ip(&interface)
    }

    async fn get_interface_mtu(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<u16, test_rpc::Error> {
        net::get_interface_mtu(&interface)
    }

    async fn get_interface_mac(
        self,
        _: context::Context,
        interface: String,
    ) -> Result<Option<[u8; 6]>, test_rpc::Error> {
        net::get_interface_mac(&interface)
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

    /// Disable the Mullvad VPN system service.
    async fn disable_mullvad_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("disable_mullvad_daemon is only implemented on Windows");
            return Err(test_rpc::Error::Syscall);
        }
        #[cfg(target_os = "windows")]
        {
            sys::disable_system_service_startup().await
        }
    }

    async fn enable_mullvad_daemon(self, _: context::Context) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("enable_mullvad_daemon is only implemented on Windows");
            return Err(test_rpc::Error::Syscall);
        }
        #[cfg(target_os = "windows")]
        {
            sys::enable_system_service_startup().await
        }
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

    async fn get_daemon_environment(
        self,
        _: context::Context,
    ) -> Result<HashMap<String, String>, test_rpc::Error> {
        sys::get_daemon_environment().await
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

    async fn spawn(self, _: context::Context, opts: SpawnOpts) -> Result<u32, test_rpc::Error> {
        let mut cmd = Command::new(&opts.path);
        cmd.args(&opts.args);

        // Make sure that PATH is updated
        // TODO: We currently do not need this on non-Windows
        #[cfg(target_os = "windows")]
        cmd.env("PATH", sys::get_system_path_var()?);

        cmd.envs(opts.env);

        if opts.attach_stdin {
            cmd.stdin(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null());
        }

        if opts.attach_stdout {
            cmd.stdout(Stdio::piped());
        }

        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = util::as_unprivileged_user(UNPRIVILEGED_USER, || cmd.spawn())
            .map_err(|error| {
                log::error!("Failed to drop privileges: {error}");
                test_rpc::Error::Syscall
            })?
            .map_err(|error| {
                log::error!("Failed to spawn {}: {error}", opts.path);
                test_rpc::Error::Syscall
            })?;

        let pid = child
            .id()
            .expect("Child hasn't been polled to completion yet");

        log::info!("spawned {} (args={:?}) (pid={pid})", opts.path, opts.args);

        let (abort_tx, abort_rx) = oneshot::channel();
        let abort_handle = || {
            let _ = abort_tx.send(());
        };

        let spawned_process = SpawnedProcess {
            stdout: child.stdout.take(),
            stdin: child.stdin.take(),
            abort_handle: OnDrop::new(Box::new(abort_handle)),
        };

        let mut state = self.0.lock().await;
        state.spawned_procs.insert(pid, spawned_process);
        drop(state);

        // spawn a task to log child stdout
        if let Some(stderr) = child.stderr.take() {
            task::spawn(async move {
                let mut stderr = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    match stderr.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end_matches(['\r', '\n']);
                            log::info!("child stderr (pid={pid}): {trimmed}");
                            line.clear();
                        }
                        Err(e) => {
                            log::error!("failed to read child stderr (pid={pid}): {e}");
                            break;
                        }
                    }
                }
            });
        }

        // spawn a task to monitor if the child exits
        task::spawn(async move {
            select! {
                result = child.wait().fuse() => match result {
                    Err(e) => {
                        log::error!("failed to await child process (pid={pid}): {e}");
                    }
                    Ok(status) => {
                        log::info!("child process (pid={pid}) exited with status: {status}");
                    }
                },

                _ = abort_rx.fuse() => {
                    if let Err(e) = child.kill().await {
                        log::error!("failed to kill child process (pid={pid}): {e}");
                    }
                }
            }

            let mut state = self.0.lock().await;
            state.spawned_procs.remove(&pid);
        });

        Ok(pid)
    }

    async fn read_child_stdout(
        self,
        _: context::Context,
        pid: u32,
    ) -> Result<Option<String>, test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .get_mut(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        let Some(stdout) = child.stdout.as_mut() else {
            return Ok(None);
        };

        let mut buf = vec![0u8; 512];

        let n = select_biased! {
            result = stdout.read(&mut buf).fuse() => result
                .map_err(|e| format!("Failed to read from child stdout: {e}"))
                .map_err(test_rpc::Error::Other)?,

            _ = sleep(Duration::from_millis(500)).fuse() => return Ok(Some(String::new())),
        };

        // check for EOF
        if n == 0 {
            child.stdout = None;
            return Ok(None);
        }

        buf.truncate(n);
        let output = String::from_utf8(buf)
            .map_err(|_| test_rpc::Error::Other("Child wrote non UTF-8 to stdout".into()))?;

        Ok(Some(output))
    }

    async fn write_child_stdin(
        self,
        _: context::Context,
        pid: u32,
        data: String,
    ) -> Result<(), test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .get_mut(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        let Some(stdin) = child.stdin.as_mut() else {
            return Err(test_rpc::Error::Other("Child stdin is closed.".into()));
        };

        stdin
            .write_all(data.as_bytes())
            .await
            .map_err(|e| format!("Error writing to child stdin: {e}"))
            .map_err(test_rpc::Error::Other)?;

        log::debug!("wrote {} bytes to pid {pid}", data.len());

        Ok(())
    }

    async fn close_child_stdin(self, _: context::Context, pid: u32) -> Result<(), test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .get_mut(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        child.stdin = None;

        Ok(())
    }

    async fn kill_child(self, _: context::Context, pid: u32) -> Result<(), test_rpc::Error> {
        let mut state = self.0.lock().await;
        let child = state
            .spawned_procs
            .remove(&pid)
            .ok_or(test_rpc::Error::UnknownPid(pid))?;

        drop(child); // I swear officer, it's not what you think!

        Ok(())
    }

    async fn get_os_version(self, _: context::Context) -> Result<OsVersion, test_rpc::Error> {
        sys::get_os_version()
    }

    #[cfg_attr(not(target_os = "macos"), expect(unused_variables))]
    async fn ifconfig_alias_add(
        self,
        _: context::Context,
        interface: String,
        alias: IpAddr,
    ) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "macos"))]
        return Err(test_rpc::Error::TargetNotImplemented);

        #[cfg(target_os = "macos")]
        talpid_macos::net::add_alias(&interface, alias)
            .await
            .map_err(|e| format!("{e:#}"))
            .map_err(test_rpc::Error::Other)
    }

    #[cfg_attr(not(target_os = "macos"), expect(unused_variables))]
    async fn ifconfig_alias_remove(
        self,
        _: context::Context,
        interface: String,
        alias: IpAddr,
    ) -> Result<(), test_rpc::Error> {
        #[cfg(not(target_os = "macos"))]
        return Err(test_rpc::Error::TargetNotImplemented);

        #[cfg(target_os = "macos")]
        talpid_macos::net::remove_alias(&interface, alias)
            .await
            .map_err(|e| format!("{e:#}"))
            .map_err(test_rpc::Error::Other)
    }
}

/// The baud rate of the serial connection between the test manager and the test runner.
/// There is a known issue with setting a baud rate at all or macOS, and the workaround
/// is to set it to zero: https://github.com/serialport/serialport-rs/pull/58
///
/// Keep this constant in sync with `test-manager/src/run_tests.rs`
const BAUD: u32 = if cfg!(target_os = "macos") { 0 } else { 115200 };

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown RPC")]
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
        server.execute(TestServer::default().serve()).await;

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
