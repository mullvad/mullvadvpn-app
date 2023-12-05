//! Manage OpenVPN tunnels.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

use crate::proxy::{ProxyMonitor, ProxyResourceData};
#[cfg(windows)]
use once_cell::sync::Lazy;
use process::openvpn::{OpenVpnCommand, OpenVpnProcHandle};
#[cfg(target_os = "linux")]
use std::collections::{HashMap, HashSet};
#[cfg(target_os = "windows")]
use std::{ffi::OsString, sync::Arc};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitStatus,
    time::Duration,
};
#[cfg(target_os = "linux")]
use talpid_routing::{self, RequiredRoute};
use talpid_tunnel::TunnelEvent;
use talpid_types::{net::openvpn, ErrorExt};
use tokio::task;

#[cfg(windows)]
use widestring::U16CString;
#[cfg(windows)]
use windows_sys::{core::GUID, Win32::NetworkManagement::Ndis::NET_LUID_LH};

#[cfg(windows)]
mod wintun;

mod mktemp;
mod process;
mod proxy;

#[cfg(windows)]
static ADAPTER_ALIAS: Lazy<U16CString> = Lazy::new(|| U16CString::from_str("Mullvad").unwrap());
#[cfg(windows)]
static ADAPTER_TUNNEL_TYPE: Lazy<U16CString> =
    Lazy::new(|| U16CString::from_str("Mullvad").unwrap());

#[cfg(windows)]
const ADAPTER_GUID: GUID = GUID {
    data1: 0xAFE43773,
    data2: 0xE1F8,
    data3: 0x4EBB,
    data4: [0x85, 0x36, 0x57, 0x6A, 0xB8, 0x6A, 0xFE, 0x9A],
};

/// Results from fallible operations on the OpenVPN tunnel.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen when using the OpenVPN tunnel.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to initialize the tokio runtime.
    #[error(display = "Failed to initialize the tokio runtime")]
    RuntimeError(#[error(source)] io::Error),

    /// Unable to start, wait for or kill the OpenVPN process.
    #[error(display = "Error in OpenVPN process management: {}", _0)]
    ChildProcessError(&'static str, #[error(source)] io::Error),

    /// Unable to start the IPC server.
    #[error(display = "Unable to start the event dispatcher IPC server")]
    EventDispatcherError(#[error(source)] event_server::Error),

    /// The OpenVPN event dispatcher exited unexpectedly
    #[error(display = "The OpenVPN event dispatcher exited unexpectedly")]
    EventDispatcherExited,

    /// cannot load wintun.dll
    #[cfg(windows)]
    #[error(display = "Failed to load wintun.dll")]
    WintunDllError(#[error(source)] io::Error),

    /// cannot create a wintun interface
    #[cfg(windows)]
    #[error(display = "Failed to create Wintun adapter")]
    WintunCreateAdapterError(#[error(source)] io::Error),

    /// OpenVPN process died unexpectedly
    #[error(display = "OpenVPN process died unexpectedly")]
    ChildProcessDied,

    /// Failed before OpenVPN started
    #[error(display = "Failed to start OpenVPN")]
    StartProcessError,

    /// The OpenVPN binary was not found.
    #[error(display = "No OpenVPN binary found at {}", _0)]
    OpenVpnNotFound(String),

    /// The OpenVPN plugin was not found.
    #[error(display = "No OpenVPN plugin found at {}", _0)]
    PluginNotFound(String),

    /// Error while writing credentials to temporary file.
    #[error(display = "Error while writing credentials to temporary file")]
    CredentialsWriteError(#[error(source)] io::Error),

    /// Failures related to the proxy service.
    #[error(display = "Proxy service failed")]
    ProxyError(#[error(source)] proxy::Error),

    /// The map is missing 'dev'
    #[cfg(target_os = "linux")]
    #[error(display = "Failed to obtain tunnel interface name")]
    MissingTunnelInterface,

    /// The map has no 'route_n' entries
    #[cfg(target_os = "linux")]
    #[error(display = "Failed to obtain OpenVPN server")]
    MissingRemoteHost,

    /// Cannot parse the remote_n in the provided map
    #[cfg(target_os = "linux")]
    #[error(display = "Cannot parse remote host string")]
    ParseRemoteHost(#[error(source)] std::net::AddrParseError),
}

#[cfg(unix)]
static OPENVPN_DIE_TIMEOUT: Duration = Duration::from_secs(4);
#[cfg(windows)]
static OPENVPN_DIE_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(target_os = "macos")]
const OPENVPN_PLUGIN_FILENAME: &str = "libtalpid_openvpn_plugin.dylib";
#[cfg(target_os = "linux")]
const OPENVPN_PLUGIN_FILENAME: &str = "libtalpid_openvpn_plugin.so";
#[cfg(windows)]
const OPENVPN_PLUGIN_FILENAME: &str = "talpid_openvpn_plugin.dll";

#[cfg(unix)]
const OPENVPN_BIN_FILENAME: &str = "openvpn";
#[cfg(windows)]
const OPENVPN_BIN_FILENAME: &str = "openvpn.exe";

/// Struct for monitoring an OpenVPN process.
#[derive(Debug)]
pub struct OpenVpnMonitor<C: OpenVpnBuilder = OpenVpnCommand> {
    prepare_task: tokio::task::JoinHandle<io::Result<C::ProcessHandle>>,

    proxy_monitor: Option<Box<dyn ProxyMonitor>>,
    /// Keep the `TempFile` for the user-pass file in the struct, so it's removed on drop.
    _user_pass_file: mktemp::TempFile,
    /// Keep the 'TempFile' for the proxy user-pass file in the struct, so it's removed on drop.
    _proxy_auth_file: Option<mktemp::TempFile>,

    event_server_abort_tx: triggered::Trigger,
    server_join_handle: task::JoinHandle<std::result::Result<(), event_server::Error>>,

    monitor_abort_tx: triggered::Trigger,
    monitor_abort_rx: triggered::Listener,

    #[cfg(windows)]
    _wintun: Arc<Box<dyn WintunContext>>,
}

#[cfg(windows)]
#[async_trait::async_trait]
trait WintunContext: Send + Sync {
    fn luid(&self) -> NET_LUID_LH;
    fn ipv6(&self) -> bool;
    async fn wait_for_interfaces(&self) -> io::Result<()>;
    fn prepare_interface(&self) {}
}

#[cfg(windows)]
impl std::fmt::Debug for dyn WintunContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WintunContext {{ luid: {}, ipv6: {} }}",
            unsafe { self.luid().Value },
            self.ipv6()
        )
    }
}

#[cfg(windows)]
#[derive(Debug)]
struct WintunContextImpl {
    adapter: wintun::WintunAdapter,
    wait_v6_interface: bool,
    _logger: wintun::WintunLoggerHandle,
}

#[cfg(windows)]
#[async_trait::async_trait]
impl WintunContext for WintunContextImpl {
    fn luid(&self) -> NET_LUID_LH {
        self.adapter.luid()
    }

    fn ipv6(&self) -> bool {
        self.wait_v6_interface
    }

    async fn wait_for_interfaces(&self) -> io::Result<()> {
        let luid = self.adapter.luid();
        talpid_windows::net::wait_for_interfaces(luid, true, self.wait_v6_interface).await
    }

    fn prepare_interface(&self) {
        self.adapter.prepare_interface();
    }
}

#[cfg(windows)]
impl WintunContextImpl {
    fn alias(&self) -> U16CString {
        self.adapter.name()
    }
}

impl OpenVpnMonitor<OpenVpnCommand> {
    /// Creates a new `OpenVpnMonitor` with the given listener and using the plugin at the given
    /// path.
    pub async fn start<L>(
        on_event: L,
        params: &openvpn::TunnelParameters,
        log_path: Option<PathBuf>,
        resource_dir: &Path,
        #[cfg(target_os = "linux")] route_manager: talpid_routing::RouteManagerHandle,
    ) -> Result<Self>
    where
        L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + 'static,
    {
        let user_pass_file =
            Self::create_credentials_file(&params.config.username, &params.config.password)
                .map_err(Error::CredentialsWriteError)?;
        let proxy_auth_file =
            Self::create_proxy_auth_file(&params.proxy).map_err(Error::CredentialsWriteError)?;
        let user_pass_file_path = user_pass_file.to_path_buf();
        let proxy_auth_file_path = proxy_auth_file.as_ref().map(|file| file.to_path_buf());

        let log_dir = log_path.as_ref().map(|log_path| {
            log_path
                .parent()
                .expect("log_path has no parent")
                .to_path_buf()
        });

        let proxy_resources = proxy::ProxyResourceData {
            resource_dir: resource_dir.to_path_buf(),
            log_dir,
        };

        let proxy_monitor = Self::start_proxy(&params.proxy, &proxy_resources).await?;

        #[cfg(windows)]
        let wintun = Self::new_wintun_context(params, resource_dir)?;

        let cmd = Self::create_openvpn_cmd(
            params,
            user_pass_file.as_ref(),
            proxy_auth_file.as_ref().map(AsRef::as_ref),
            resource_dir,
            &proxy_monitor,
            #[cfg(windows)]
            wintun.alias().to_os_string(),
        )?;

        let plugin_path = Self::get_plugin_path(resource_dir)?;

        #[cfg(target_os = "linux")]
        let ipv6_enabled = params.generic_options.enable_ipv6;

        let (event_server_abort_tx, event_server_abort_rx) = triggered::trigger();

        let openvpn_init_args = OpenVpnTunnelInitArgs {
            event_server_abort_tx: event_server_abort_tx.clone(),
            event_server_abort_rx,
            plugin_path,
            log_path,
            user_pass_file,
            proxy_auth_file,
            proxy_monitor,
            #[cfg(target_os = "linux")]
            fwmark: params.fwmark,
        };
        Self::new_internal(
            cmd,
            openvpn_init_args,
            event_server::OpenvpnEventProxyImpl {
                on_event,
                user_pass_file_path: user_pass_file_path.clone(),
                proxy_auth_file_path: proxy_auth_file_path.clone(),
                abort_server_tx: event_server_abort_tx,
                proxy: params.proxy.clone(),
                #[cfg(target_os = "linux")]
                route_manager_handle: route_manager,
                #[cfg(target_os = "linux")]
                ipv6_enabled,
            },
            #[cfg(windows)]
            Box::new(wintun),
        )
    }

    #[cfg(windows)]
    fn new_wintun_context(
        params: &openvpn::TunnelParameters,
        resource_dir: &Path,
    ) -> Result<WintunContextImpl> {
        let dll = wintun::WintunDll::instance(resource_dir).map_err(Error::WintunDllError)?;
        let wintun_logger = dll.activate_logging();

        let wintun_adapter = wintun::WintunAdapter::create(
            dll,
            &ADAPTER_ALIAS,
            &ADAPTER_TUNNEL_TYPE,
            Some(ADAPTER_GUID),
        )
        .map_err(Error::WintunCreateAdapterError)?;

        Ok(WintunContextImpl {
            adapter: wintun_adapter,
            wait_v6_interface: params.generic_options.enable_ipv6,
            _logger: wintun_logger,
        })
    }
}

#[cfg(target_os = "linux")]
fn extract_routes(env: &HashMap<String, String>) -> Result<HashSet<RequiredRoute>> {
    let tun_interface = env.get("dev").ok_or(Error::MissingTunnelInterface)?;
    let tun_node = talpid_routing::Node::device(tun_interface.to_string());
    let mut routes = HashSet::new();
    for network in &["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()] {
        routes.insert(RequiredRoute::new(*network, tun_node.clone()).use_main_table(false));
    }
    Ok(routes)
}

struct OpenVpnTunnelInitArgs {
    event_server_abort_tx: triggered::Trigger,
    event_server_abort_rx: triggered::Listener,
    plugin_path: PathBuf,
    log_path: Option<PathBuf>,
    user_pass_file: mktemp::TempFile,
    proxy_auth_file: Option<mktemp::TempFile>,
    proxy_monitor: Option<Box<dyn ProxyMonitor>>,
    #[cfg(target_os = "linux")]
    fwmark: u32,
}

impl<C: OpenVpnBuilder + Send + 'static> OpenVpnMonitor<C> {
    fn new_internal<L>(
        mut cmd: C,
        init_args: OpenVpnTunnelInitArgs,
        on_event: L,
        #[cfg(windows)] wintun: Box<dyn WintunContext>,
    ) -> Result<OpenVpnMonitor<C>>
    where
        L: event_server::OpenvpnEventProxy + Send + Sync + 'static,
    {
        let event_server_abort_tx = init_args.event_server_abort_tx;
        let event_server_abort_rx = init_args.event_server_abort_rx;
        let plugin_path = init_args.plugin_path;
        let log_path = init_args.log_path;
        let user_pass_file = init_args.user_pass_file;
        let proxy_auth_file = init_args.proxy_auth_file;
        let proxy_monitor = init_args.proxy_monitor;

        let (server_join_handle, ipc_path) = event_server::start(on_event, event_server_abort_rx)
            .map_err(Error::EventDispatcherError)?;

        #[cfg(windows)]
        let wintun = Arc::new(wintun);

        #[cfg(target_os = "linux")]
        cmd.fwmark(init_args.fwmark);

        cmd.plugin(plugin_path, vec![ipc_path])
            .log(log_path.as_deref());
        let prepare_task = tokio::spawn(Self::prepare_process(
            cmd,
            #[cfg(windows)]
            wintun.clone(),
        ));

        let (monitor_abort_tx, monitor_abort_rx) = triggered::trigger();

        let monitor = OpenVpnMonitor {
            prepare_task,
            proxy_monitor,
            _user_pass_file: user_pass_file,
            _proxy_auth_file: proxy_auth_file,

            event_server_abort_tx,
            server_join_handle,

            monitor_abort_tx,
            monitor_abort_rx,

            #[cfg(windows)]
            _wintun: wintun,
        };

        Ok(monitor)
    }

    #[cfg_attr(not(windows), allow(clippy::unused_async))]
    async fn prepare_process(
        cmd: C,
        #[cfg(windows)] wintun: Arc<Box<dyn WintunContext>>,
    ) -> io::Result<C::ProcessHandle> {
        #[cfg(windows)]
        {
            log::debug!("Wait for IP interfaces");
            wintun.wait_for_interfaces().await?;
            wintun.prepare_interface();
        }
        cmd.start()
    }

    /// Creates a handle to this monitor, allowing the tunnel to be closed while some other
    /// thread is blocked in `wait`.
    pub fn close_handle(&self) -> OpenVpnCloseHandle {
        OpenVpnCloseHandle {
            monitor_abort_tx: self.monitor_abort_tx.clone(),
            prepare_task: self.prepare_task.abort_handle(),
        }
    }

    /// Consumes the monitor and waits for both proxy and tunnel, as applicable.
    pub async fn wait(mut self) -> Result<()> {
        if let Some(mut proxy_monitor) = self.proxy_monitor.take() {
            let tunnel_close_handle = self.close_handle();
            let proxy_close_handle = proxy_monitor.close_handle();

            let tunnel_task = async move {
                let result = self.wait_tunnel().await;
                let _ = proxy_close_handle.close();
                result
            };

            let proxy_task = async move {
                let result = proxy_monitor.wait().await;
                tunnel_close_handle.close();
                result.map_err(Error::ProxyError)
            };

            join_return_first(tunnel_task, proxy_task).await
        } else {
            // No proxy active, wait only for the tunnel.
            self.wait_tunnel().await
        }
    }

    /// Supplement `inner_wait_tunnel()` with logging and error handling.
    async fn wait_tunnel(self) -> Result<()> {
        match self.inner_wait_tunnel().await {
            WaitResult::Preparation(result) => match result {
                Err(error) => {
                    log::debug!(
                        "{}",
                        error.display_chain_with_msg("Failed to start OpenVPN")
                    );
                    Err(Error::StartProcessError)
                }
                _ => Ok(()),
            },
            WaitResult::Child(Ok(exit_status)) => {
                if exit_status.success() {
                    log::debug!(
                        "OpenVPN exited, as expected, with exit status: {}",
                        exit_status
                    );
                    Ok(())
                } else {
                    log::error!("OpenVPN died unexpectedly with status: {}", exit_status);
                    Err(Error::ChildProcessDied)
                }
            }
            WaitResult::Child(Err(e)) => {
                log::error!("OpenVPN process wait error: {}", e);
                Err(Error::ChildProcessError("Error when waiting", e))
            }
            WaitResult::EventDispatcher => {
                log::error!("OpenVPN Event server exited unexpectedly");
                Err(Error::EventDispatcherExited)
            }
        }
    }

    /// Waits for both the child process and the event dispatcher in parallel. After both have
    /// returned this returns the earliest result.
    async fn inner_wait_tunnel(self) -> WaitResult {
        let mut child = match self.prepare_task.await {
            Ok(Ok(child)) => child,
            Ok(Err(error)) => {
                return WaitResult::Preparation(Err(error));
            }
            Err(_) => return WaitResult::Preparation(Ok(())),
        };

        let kill_child = async move {
            let result = tokio::select! {
                result = child.wait() => {
                    log::debug!("OpenVPN process exited");
                    result
                }
                _ = self.monitor_abort_rx => {
                    log::debug!("Killing OpenVPN process");
                    child.kill();
                    child.wait().await
                }
            };

            self.event_server_abort_tx.trigger();
            WaitResult::Child(result)
        };
        let kill_event_dispatcher = async move {
            let _ = self.server_join_handle.await;
            WaitResult::EventDispatcher
        };

        join_return_first(kill_child, kill_event_dispatcher).await
    }

    fn create_proxy_auth_file(
        proxy_settings: &Option<openvpn::ProxySettings>,
    ) -> std::result::Result<Option<mktemp::TempFile>, io::Error> {
        if let Some(openvpn::ProxySettings::Remote(ref remote_proxy)) = proxy_settings {
            if let Some(ref proxy_auth) = remote_proxy.auth {
                return Ok(Some(Self::create_credentials_file(
                    &proxy_auth.username,
                    &proxy_auth.password,
                )?));
            }
        }
        Ok(None)
    }

    /// Starts a proxy service, as applicable.
    async fn start_proxy(
        proxy_settings: &Option<openvpn::ProxySettings>,
        proxy_resources: &ProxyResourceData,
    ) -> Result<Option<Box<dyn ProxyMonitor>>> {
        if let Some(ref settings) = proxy_settings {
            let proxy_monitor = proxy::start_proxy(settings, proxy_resources)
                .await
                .map_err(Error::ProxyError)?;
            return Ok(Some(proxy_monitor));
        }
        Ok(None)
    }

    fn create_credentials_file(username: &str, password: &str) -> io::Result<mktemp::TempFile> {
        let temp_file = mktemp::TempFile::new();
        log::debug!("Writing credentials to {}", temp_file.as_ref().display());
        let mut file = fs::File::create(&temp_file)?;
        Self::set_user_pass_file_permissions(&file)?;
        write!(file, "{username}\n{password}\n")?;
        Ok(temp_file)
    }

    #[cfg(unix)]
    fn set_user_pass_file_permissions(file: &fs::File) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(PermissionsExt::from_mode(0o400))
    }

    #[cfg(windows)]
    fn set_user_pass_file_permissions(_file: &fs::File) -> io::Result<()> {
        // TODO(linus): Lock permissions correctly on Windows.
        Ok(())
    }

    fn get_plugin_path(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_PLUGIN_FILENAME);
        if path.exists() {
            log::trace!("Using OpenVPN plugin at {}", path.display());
            Ok(path)
        } else {
            Err(Error::PluginNotFound(path.display().to_string()))
        }
    }

    fn create_openvpn_cmd(
        params: &openvpn::TunnelParameters,
        user_pass_file: &Path,
        proxy_auth_file: Option<&Path>,
        resource_dir: &Path,
        proxy_monitor: &Option<Box<dyn ProxyMonitor>>,
        #[cfg(windows)] alias: OsString,
    ) -> Result<OpenVpnCommand> {
        let mut cmd = OpenVpnCommand::new(Self::get_openvpn_bin(resource_dir)?);
        if let Some(config) = Self::get_config_path(resource_dir) {
            cmd.config(config);
        }
        cmd.remote(params.config.endpoint)
            .user_pass(user_pass_file)
            .tunnel_options(&params.options)
            .enable_ipv6(params.generic_options.enable_ipv6)
            .ca(resource_dir.join("ca.crt"));
        #[cfg(windows)]
        cmd.tunnel_alias(Some(alias));
        if let Some(proxy_settings) = params.proxy.clone().take() {
            cmd.proxy_settings(proxy_settings);
        }
        if let Some(proxy_auth_file) = proxy_auth_file {
            cmd.proxy_auth(proxy_auth_file);
        }
        if let Some(proxy) = proxy_monitor {
            cmd.proxy_port(proxy.port());
        }

        Ok(cmd)
    }

    fn get_openvpn_bin(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_BIN_FILENAME);
        if path.exists() {
            log::trace!("Using OpenVPN at {}", path.display());
            Ok(path)
        } else {
            Err(Error::OpenVpnNotFound(path.display().to_string()))
        }
    }

    fn get_config_path(resource_dir: &Path) -> Option<PathBuf> {
        let path = resource_dir.join("openvpn.conf");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

/// A handle to an `OpenVpnMonitor` for closing it.
#[derive(Debug)]
pub struct OpenVpnCloseHandle {
    monitor_abort_tx: triggered::Trigger,
    prepare_task: tokio::task::AbortHandle,
}

impl OpenVpnCloseHandle {
    /// Begin killing the OpenVPN monitor, making the `OpenVpnMonitor::wait` method return.
    pub fn close(self) {
        self.prepare_task.abort();
        self.monitor_abort_tx.trigger();
    }
}

/// Internal enum to differentiate between if the child process or the event dispatcher died first.
#[derive(Debug)]
enum WaitResult {
    Preparation(io::Result<()>),
    Child(io::Result<ExitStatus>),
    EventDispatcher,
}

/// Trait for types acting as OpenVPN process starters for `OpenVpnMonitor`.
pub trait OpenVpnBuilder {
    /// The type of handles to subprocesses this builder produces.
    type ProcessHandle: ProcessHandle;

    /// Set the OpenVPN plugin to the given values.
    fn plugin(&mut self, path: impl AsRef<Path>, args: Vec<String>) -> &mut Self;

    /// Set the OpenVPN log file path to use.
    fn log(&mut self, log_path: Option<impl AsRef<Path>>) -> &mut Self;

    /// Spawn the subprocess and return a handle.
    fn start(&self) -> io::Result<Self::ProcessHandle>;

    /// Sets the firewall mark for the connection.
    #[cfg(target_os = "linux")]
    fn fwmark(&mut self, fwmark: u32) -> &mut Self;
}

/// Trait for types acting as handles to subprocesses for `OpenVpnMonitor`
#[async_trait::async_trait]
pub trait ProcessHandle: Send + Sync + 'static {
    /// Block until the subprocess exits or there is an error in the wait syscall.
    async fn wait(&mut self) -> io::Result<ExitStatus>;

    /// Kill the subprocess without waiting for it to complete.
    fn kill(&mut self);
}

impl OpenVpnBuilder for OpenVpnCommand {
    type ProcessHandle = OpenVpnProcHandle;

    fn plugin(&mut self, path: impl AsRef<Path>, args: Vec<String>) -> &mut Self {
        self.plugin(path, args)
    }

    fn log(&mut self, log_path: Option<impl AsRef<Path>>) -> &mut Self {
        if let Some(log_path) = log_path {
            self.log(log_path)
        } else {
            self
        }
    }

    fn start(&self) -> io::Result<OpenVpnProcHandle> {
        OpenVpnProcHandle::new(&mut self.build())
    }

    #[cfg(target_os = "linux")]
    fn fwmark(&mut self, fwmark: u32) -> &mut Self {
        self.fwmark(Some(fwmark));
        self
    }
}

#[async_trait::async_trait]
impl ProcessHandle for OpenVpnProcHandle {
    async fn wait(&mut self) -> io::Result<ExitStatus> {
        OpenVpnProcHandle::wait(self).await
    }

    fn kill(&mut self) {
        OpenVpnProcHandle::kill(self, OPENVPN_DIE_TIMEOUT)
    }
}

/// Join two futures and return the result of the first one to complete.
async fn join_return_first<R>(
    future1: impl std::future::Future<Output = R>,
    future2: impl std::future::Future<Output = R>,
) -> R {
    futures::pin_mut!(future1);
    futures::pin_mut!(future2);

    match futures::future::select(future1, future2).await {
        futures::future::Either::Left((result, other)) => {
            let _ = other.await;
            result
        }
        futures::future::Either::Right((result, other)) => {
            let _ = other.await;
            result
        }
    }
}

mod event_server {
    use futures::stream::TryStreamExt;
    use parity_tokio_ipc::Endpoint as IpcEndpoint;
    use std::{
        collections::HashMap,
        pin::Pin,
        task::{Context, Poll},
    };
    use talpid_tunnel::TunnelMetadata;
    #[cfg(any(target_os = "linux", windows))]
    use talpid_types::ErrorExt;
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
    use tonic::{
        self,
        transport::{server::Connected, Server},
        Request, Response,
    };

    #[allow(clippy::derive_partial_eq_without_eq)]
    mod proto {
        tonic::include_proto!("talpid_openvpn_plugin");
    }
    pub use proto::{
        openvpn_event_proxy_server::{OpenvpnEventProxy, OpenvpnEventProxyServer},
        EventDetails,
    };

    #[derive(err_derive::Error, Debug)]
    pub enum Error {
        /// Failure to set up the IPC server.
        #[error(display = "Failed to create pipe or Unix socket")]
        StartServer(#[error(source)] std::io::Error),

        /// An error occurred while the server was running.
        #[error(display = "Tonic error")]
        TonicError(#[error(source)] tonic::transport::Error),
    }

    /// Implements a gRPC service used to process events sent to by OpenVPN.
    pub struct OpenvpnEventProxyImpl<
        L: (Fn(
                talpid_tunnel::TunnelEvent,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + 'static,
    > {
        pub on_event: L,
        pub user_pass_file_path: super::PathBuf,
        pub proxy_auth_file_path: Option<super::PathBuf>,
        pub abort_server_tx: triggered::Trigger,
        pub proxy: Option<talpid_types::net::openvpn::ProxySettings>,
        #[cfg(target_os = "linux")]
        pub route_manager_handle: talpid_routing::RouteManagerHandle,
        #[cfg(target_os = "linux")]
        pub ipv6_enabled: bool,
    }

    impl<
            L: (Fn(
                    talpid_tunnel::TunnelEvent,
                )
                    -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
                + Send
                + Sync
                + 'static,
        > OpenvpnEventProxyImpl<L>
    {
        async fn up_inner(
            &self,
            request: Request<EventDetails>,
        ) -> std::result::Result<Response<()>, tonic::Status> {
            let env = request.into_inner().env;
            (self.on_event)(talpid_tunnel::TunnelEvent::InterfaceUp(
                Self::get_tunnel_metadata(&env)?,
                talpid_types::net::AllowedTunnelTraffic::All,
            ))
            .await;
            Ok(Response::new(()))
        }

        async fn route_up_inner(
            &self,
            request: Request<EventDetails>,
        ) -> std::result::Result<Response<()>, tonic::Status> {
            let env = request.into_inner().env;

            let _ = tokio::fs::remove_file(&self.user_pass_file_path).await;
            if let Some(ref file_path) = &self.proxy_auth_file_path {
                let _ = tokio::fs::remove_file(file_path).await;
            }

            #[cfg(target_os = "linux")]
            {
                let route_handle = self.route_manager_handle.clone();
                let ipv6_enabled = self.ipv6_enabled;

                let mut routes: std::collections::HashSet<_, _> = super::extract_routes(&env)
                    .map_err(|err| {
                        log::error!("{}", err.display_chain_with_msg("Failed to obtain routes"));
                        tonic::Status::failed_precondition("Failed to obtain routes")
                    })?
                    .into_iter()
                    .filter(|route| route.prefix.is_ipv4() || ipv6_enabled)
                    .collect();

                if let Some(proxy_settings) = &self.proxy {
                    if let talpid_types::net::openvpn::ProxySettings::Local(proxy_settings) = proxy_settings {
                        let network = proxy_settings.peer.ip().into();
                        let node = talpid_routing::Node::new("192.168.1.1".parse().unwrap(), String::from("wlp0s20f3"));
                        routes.insert(talpid_routing::RequiredRoute::new(network, node).use_main_table(false));
                    }
                }

                if let Err(error) = route_handle.add_routes(routes).await {
                    log::error!("{}", error.display_chain());
                    return Err(tonic::Status::failed_precondition("Failed to add routes"));
                }
                if let Err(error) = route_handle.create_routing_rules(ipv6_enabled).await {
                    log::error!("{}", error.display_chain());
                    return Err(tonic::Status::failed_precondition("Failed to add routes"));
                }
            }

            let metadata = Self::get_tunnel_metadata(&env)?;

            #[cfg(windows)]
            {
                let tunnel_device = metadata.interface.clone();
                let luid =
                    talpid_windows::net::luid_from_alias(tunnel_device).map_err(|error| {
                        log::error!("{}", error.display_chain_with_msg("luid_from_alias failed"));
                        tonic::Status::unavailable("failed to obtain interface luid")
                    })?;
                talpid_windows::net::wait_for_addresses(luid)
                    .await
                    .map_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("wait_for_addresses failed")
                        );
                        tonic::Status::unavailable("wait_for_addresses failed")
                    })?;
            }

            (self.on_event)(talpid_tunnel::TunnelEvent::Up(metadata)).await;

            Ok(Response::new(()))
        }

        fn get_tunnel_metadata(
            env: &HashMap<String, String>,
        ) -> std::result::Result<TunnelMetadata, tonic::Status> {
            let tunnel_alias = env
                .get("dev")
                .ok_or_else(|| tonic::Status::invalid_argument("missing tunnel alias"))?
                .to_string();

            let mut ips = vec![env
                .get("ifconfig_local")
                .ok_or_else(|| {
                    tonic::Status::invalid_argument("missing \"ifconfig_local\" in up event")
                })?
                .parse()
                .map_err(|_| tonic::Status::invalid_argument("Invalid tunnel IPv4 address"))?];
            if let Some(ipv6_address) = env.get("ifconfig_ipv6_local") {
                ips.push(
                    ipv6_address.parse().map_err(|_| {
                        tonic::Status::invalid_argument("Invalid tunnel IPv6 address")
                    })?,
                );
            }
            let ipv4_gateway = env
                .get("route_vpn_gateway")
                .ok_or_else(|| {
                    tonic::Status::invalid_argument("No \"route_vpn_gateway\" in tunnel up event")
                })?
                .parse()
                .map_err(|_| {
                    tonic::Status::invalid_argument("Invalid tunnel gateway IPv4 address")
                })?;
            let ipv6_gateway = if let Some(ipv6_address) = env.get("route_ipv6_gateway_1") {
                Some(ipv6_address.parse().map_err(|_| {
                    tonic::Status::invalid_argument("Invalid tunnel gateway IPv6 address")
                })?)
            } else {
                None
            };

            Ok(TunnelMetadata {
                interface: tunnel_alias,
                ips,
                ipv4_gateway,
                ipv6_gateway,
            })
        }
    }

    #[tonic::async_trait]
    impl<
            L: (Fn(
                    talpid_tunnel::TunnelEvent,
                )
                    -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
                + Send
                + Sync
                + 'static,
        > OpenvpnEventProxy for OpenvpnEventProxyImpl<L>
    {
        async fn auth_failed(
            &self,
            request: Request<EventDetails>,
        ) -> std::result::Result<Response<()>, tonic::Status> {
            let env = request.into_inner().env;
            (self.on_event)(talpid_tunnel::TunnelEvent::AuthFailed(
                env.get("auth_failed_reason").cloned(),
            ))
            .await;
            Ok(Response::new(()))
        }

        async fn up(
            &self,
            request: Request<EventDetails>,
        ) -> std::result::Result<Response<()>, tonic::Status> {
            self.up_inner(request).await.map_err(|error| {
                self.abort_server_tx.trigger();
                error
            })
        }

        async fn route_up(
            &self,
            request: Request<EventDetails>,
        ) -> std::result::Result<Response<()>, tonic::Status> {
            self.route_up_inner(request).await.map_err(|error| {
                self.abort_server_tx.trigger();
                error
            })
        }

        async fn route_predown(
            &self,
            _request: Request<EventDetails>,
        ) -> std::result::Result<Response<()>, tonic::Status> {
            (self.on_event)(talpid_tunnel::TunnelEvent::Down).await;
            Ok(Response::new(()))
        }
    }

    pub fn start<L>(
        event_proxy: L,
        abort_rx: triggered::Listener,
    ) -> std::result::Result<(tokio::task::JoinHandle<Result<(), Error>>, String), Error>
    where
        L: OpenvpnEventProxy + Sync + Send + 'static,
    {
        let uuid = uuid::Uuid::new_v4().to_string();
        let ipc_path = if cfg!(windows) {
            format!("//./pipe/talpid-openvpn-{uuid}")
        } else {
            format!("/tmp/talpid-openvpn-{uuid}")
        };

        let endpoint = IpcEndpoint::new(ipc_path.clone());
        let incoming = endpoint.incoming().map_err(Error::StartServer)?;
        Ok((
            tokio::spawn(async move {
                Server::builder()
                    .add_service(OpenvpnEventProxyServer::new(event_proxy))
                    .serve_with_incoming_shutdown(incoming.map_ok(StreamBox), abort_rx)
                    .await
                    .map_err(Error::TonicError)
            }),
            ipc_path,
        ))
    }

    #[derive(Debug)]
    pub struct StreamBox<T: AsyncRead + AsyncWrite>(pub T);
    impl<T: AsyncRead + AsyncWrite> Connected for StreamBox<T> {
        type ConnectInfo = Option<()>;

        fn connect_info(&self) -> Self::ConnectInfo {
            None
        }
    }
    impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamBox<T> {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.0).poll_read(cx, buf)
        }
    }
    impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamBox<T> {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Pin::new(&mut self.0).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }

        fn poll_shutdown(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.0).poll_shutdown(cx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mktemp::TempFile;
    use std::{
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    #[cfg(windows)]
    #[derive(Debug)]
    struct TestWintunContext {}

    #[cfg(windows)]
    #[async_trait::async_trait]
    impl WintunContext for TestWintunContext {
        fn luid(&self) -> NET_LUID_LH {
            NET_LUID_LH { Value: 0u64 }
        }
        fn ipv6(&self) -> bool {
            false
        }
        async fn wait_for_interfaces(&self) -> io::Result<()> {
            Ok(())
        }
    }

    struct TestOpenvpnEventProxy {}

    #[async_trait::async_trait]
    impl event_server::OpenvpnEventProxy for TestOpenvpnEventProxy {
        async fn auth_failed(
            &self,
            _request: tonic::Request<event_server::EventDetails>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            Ok(tonic::Response::new(()))
        }
        async fn up(
            &self,
            _request: tonic::Request<event_server::EventDetails>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            Ok(tonic::Response::new(()))
        }
        async fn route_up(
            &self,
            _request: tonic::Request<event_server::EventDetails>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            Ok(tonic::Response::new(()))
        }
        async fn route_predown(
            &self,
            _request: tonic::Request<event_server::EventDetails>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            Ok(tonic::Response::new(()))
        }
    }

    #[derive(Debug, Default, Clone)]
    struct TestOpenVpnBuilder {
        pub plugin: Arc<Mutex<Option<PathBuf>>>,
        pub log: Arc<Mutex<Option<PathBuf>>>,
        pub process_handle: Option<TestProcessHandle>,
    }

    impl OpenVpnBuilder for TestOpenVpnBuilder {
        type ProcessHandle = TestProcessHandle;

        fn plugin(&mut self, path: impl AsRef<Path>, _args: Vec<String>) -> &mut Self {
            *self.plugin.lock().unwrap() = Some(path.as_ref().to_path_buf());
            self
        }

        fn log(&mut self, log: Option<impl AsRef<Path>>) -> &mut Self {
            *self.log.lock().unwrap() = log.as_ref().map(|path| path.as_ref().to_path_buf());
            self
        }

        #[cfg(target_os = "linux")]
        fn fwmark(&mut self, _fwmark: u32) -> &mut Self {
            self
        }

        fn start(&self) -> io::Result<Self::ProcessHandle> {
            self.process_handle
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to start"))
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct TestProcessHandle(i32);

    #[async_trait::async_trait]
    impl ProcessHandle for TestProcessHandle {
        #[cfg(unix)]
        async fn wait(&mut self) -> io::Result<ExitStatus> {
            use std::os::unix::process::ExitStatusExt;
            Ok(ExitStatus::from_raw(self.0))
        }

        #[cfg(windows)]
        async fn wait(&mut self) -> io::Result<ExitStatus> {
            use std::os::windows::process::ExitStatusExt;
            Ok(ExitStatus::from_raw(self.0 as u32))
        }

        fn kill(&mut self) {}
    }

    fn create_init_args_plugin_log(
        plugin_path: PathBuf,
        log_path: Option<PathBuf>,
    ) -> OpenVpnTunnelInitArgs {
        let (event_server_abort_tx, event_server_abort_rx) = triggered::trigger();
        OpenVpnTunnelInitArgs {
            event_server_abort_tx,
            event_server_abort_rx,
            plugin_path,
            log_path,
            user_pass_file: TempFile::new(),
            proxy_auth_file: None,
            proxy_monitor: None,
            #[cfg(target_os = "linux")]
            fwmark: 0,
        }
    }

    fn create_init_args() -> OpenVpnTunnelInitArgs {
        create_init_args_plugin_log("".into(), None)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sets_plugin() {
        let builder = TestOpenVpnBuilder::default();
        let openvpn_init_args = create_init_args_plugin_log("./my_test_plugin".into(), None);
        let _ = OpenVpnMonitor::new_internal(
            builder.clone(),
            openvpn_init_args,
            TestOpenvpnEventProxy {},
            #[cfg(windows)]
            Box::new(TestWintunContext {}),
        );
        assert_eq!(
            Some(PathBuf::from("./my_test_plugin")),
            *builder.plugin.lock().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sets_log() {
        let builder = TestOpenVpnBuilder::default();
        let openvpn_init_args =
            create_init_args_plugin_log("".into(), Some(PathBuf::from("./my_test_log_file")));
        let _ = OpenVpnMonitor::new_internal(
            builder.clone(),
            openvpn_init_args,
            TestOpenvpnEventProxy {},
            #[cfg(windows)]
            Box::new(TestWintunContext {}),
        );
        assert_eq!(
            Some(PathBuf::from("./my_test_log_file")),
            *builder.log.lock().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn exit_successfully() {
        let builder = TestOpenVpnBuilder {
            process_handle: Some(TestProcessHandle(0)),
            ..Default::default()
        };
        let openvpn_init_args = create_init_args();
        let testee = OpenVpnMonitor::new_internal(
            builder,
            openvpn_init_args,
            TestOpenvpnEventProxy {},
            #[cfg(windows)]
            Box::new(TestWintunContext {}),
        )
        .unwrap();
        assert!(testee.wait().await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn exit_error() {
        let builder = TestOpenVpnBuilder {
            process_handle: Some(TestProcessHandle(1)),
            ..Default::default()
        };
        let openvpn_init_args = create_init_args();
        let testee = OpenVpnMonitor::new_internal(
            builder,
            openvpn_init_args,
            TestOpenvpnEventProxy {},
            #[cfg(windows)]
            Box::new(TestWintunContext {}),
        )
        .unwrap();
        assert!(testee.wait().await.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn wait_closed() {
        let builder = TestOpenVpnBuilder {
            process_handle: Some(TestProcessHandle(1)),
            ..Default::default()
        };
        let openvpn_init_args = create_init_args();
        let testee = OpenVpnMonitor::new_internal(
            builder,
            openvpn_init_args,
            TestOpenvpnEventProxy {},
            #[cfg(windows)]
            Box::new(TestWintunContext {}),
        )
        .unwrap();

        testee.close_handle().close();
        let result = testee.wait().await;
        println!("[testee.wait(): {:?}]", result);
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn failed_process_start() {
        let builder = TestOpenVpnBuilder::default();
        let openvpn_init_args = create_init_args();
        let result = OpenVpnMonitor::new_internal(
            builder,
            openvpn_init_args,
            TestOpenvpnEventProxy {},
            #[cfg(windows)]
            Box::new(TestWintunContext {}),
        )
        .unwrap();
        match result.wait().await {
            Err(Error::StartProcessError) => (),
            _ => panic!("Wrong error"),
        }
    }
}
