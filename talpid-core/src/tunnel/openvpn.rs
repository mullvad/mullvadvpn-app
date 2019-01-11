use super::TunnelEvent;
use crate::{
    mktemp,
    process::{
        openvpn::{OpenVpnCommand, OpenVpnProcHandle},
        stoppable_process::StoppableProcess,
    },
};
#[cfg(target_os = "linux")]
use failure::ResultExt as FailureResultExt;
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitStatus,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
use talpid_ipc;
use talpid_types::net::openvpn;
#[cfg(target_os = "linux")]
use which;

error_chain! {
    errors {
        /// Unable to start, wait for or kill the OpenVPN process.
        ChildProcessError(msg: &'static str) {
            description("Unable to start, wait for or kill the OpenVPN process")
            display("OpenVPN process error: {}", msg)
        }
        /// Unable to start or manage the IPC server listening for events from OpenVPN.
        EventDispatcherError {
            description("Unable to start or manage the event dispatcher IPC server")
        }
        #[cfg(windows)]
        /// No TAP adapter was detected
        MissingTapAdapter {
            description("No TAP adapter was detected")
        }
        #[cfg(windows)]
        /// TAP adapter seems to be disabled
        DisabledTapAdapter {
            description("The TAP adapter appears to be disabled")
        }
        /// The IP routing program was not found.
        #[cfg(target_os = "linux")]
        IpRouteNotFound {
            description("The IP routing program `ip` was not found.")
        }
        /// The OpenVPN binary was not found.
        OpenVpnNotFound(path: PathBuf) {
            description("No OpenVPN binary found")
            display("No OpenVPN binary found at {}", path.display())
        }
        /// The OpenVPN plugin was not found.
        PluginNotFound(path: PathBuf) {
            description("No OpenVPN plugin found")
            display("No OpenVPN plugin found at {}", path.display())
        }
        /// There was an error when writing authentication credentials to temporary file.
        CredentialsWriteError {
            description("Error while writing credentials to temporary file")
        }
    }
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
    child: Arc<C::ProcessHandle>,
    event_dispatcher: Option<talpid_ipc::IpcServer>,
    log_path: Option<PathBuf>,
    closed: Arc<AtomicBool>,
    /// Keep the `TempFile` for the user-pass file in the struct, so it's removed on drop.
    _user_pass_file: mktemp::TempFile,
    /// Keep the 'TempFile' for the proxy user-pass file in the struct, so it's removed on drop.
    _proxy_auth_file: Option<mktemp::TempFile>,
}

impl OpenVpnMonitor<OpenVpnCommand> {
    /// Creates a new `OpenVpnMonitor` with the given listener and using the plugin at the given
    /// path.
    pub fn start<L>(
        on_event: L,
        params: &openvpn::TunnelParameters,
        tunnel_alias: Option<OsString>,
        log_path: Option<PathBuf>,
        resource_dir: &Path,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        let user_pass_file =
            Self::create_credentials_file(&params.config.username, &params.config.password)
                .chain_err(|| ErrorKind::CredentialsWriteError)?;

        let proxy_auth_file = Self::create_proxy_auth_file(&params.options.proxy)
            .chain_err(|| ErrorKind::CredentialsWriteError)?;


        let user_pass_file_path = user_pass_file.to_path_buf();

        let proxy_auth_file_path = match proxy_auth_file {
            Some(ref file) => Some(file.to_path_buf()),
            _ => None,
        };

        let on_openvpn_event = move |event, env| {
            if event == openvpn_plugin::EventType::RouteUp {
                // The user-pass file has been read. Try to delete it early.
                let _ = fs::remove_file(&user_pass_file_path);

                // The proxy auth file has been read. Try to delete it early.
                if let Some(ref file_path) = &proxy_auth_file_path {
                    let _ = fs::remove_file(file_path);
                }
            }
            match TunnelEvent::from_openvpn_event(event, &env) {
                Some(tunnel_event) => on_event(tunnel_event),
                None => log::debug!("Ignoring OpenVpnEvent {:?}", event),
            }
        };
        let cmd = Self::create_openvpn_cmd(
            params,
            tunnel_alias,
            user_pass_file.as_ref(),
            match proxy_auth_file {
                Some(ref file) => Some(file.as_ref()),
                _ => None,
            },
            resource_dir,
        )?;

        let plugin_path = Self::get_plugin_path(resource_dir)?;

        Self::new_internal(
            cmd,
            on_openvpn_event,
            &plugin_path,
            log_path,
            user_pass_file,
            proxy_auth_file,
        )
    }
}

impl<C: OpenVpnBuilder> OpenVpnMonitor<C> {
    fn new_internal<L>(
        mut cmd: C,
        on_event: L,
        plugin_path: impl AsRef<Path>,
        log_path: Option<PathBuf>,
        user_pass_file: mktemp::TempFile,
        proxy_auth_file: Option<mktemp::TempFile>,
    ) -> Result<OpenVpnMonitor<C>>
    where
        L: Fn(openvpn_plugin::EventType, HashMap<String, String>) + Send + Sync + 'static,
    {
        let event_dispatcher =
            event_server::start(on_event).chain_err(|| ErrorKind::EventDispatcherError)?;


        let child = cmd
            .plugin(plugin_path, vec![event_dispatcher.path().to_owned()])
            .log(log_path.as_ref().map(|p| p.as_path()))
            .start()
            .chain_err(|| ErrorKind::ChildProcessError("Failed to start"))?;


        Ok(OpenVpnMonitor {
            child: Arc::new(child),
            event_dispatcher: Some(event_dispatcher),
            log_path,
            closed: Arc::new(AtomicBool::new(false)),
            _user_pass_file: user_pass_file,
            _proxy_auth_file: proxy_auth_file,
        })
    }

    /// Creates a handle to this monitor, allowing the tunnel to be closed while some other
    /// thread
    /// is blocked in `wait`.
    pub fn close_handle(&self) -> OpenVpnCloseHandle<C::ProcessHandle> {
        OpenVpnCloseHandle {
            child: self.child.clone(),
            closed: self.closed.clone(),
        }
    }

    /// Consumes the monitor and blocks until OpenVPN exits or there is an error in either
    /// waiting
    /// for the process or in the event dispatcher.
    pub fn wait(mut self) -> Result<()> {
        match self.wait_result() {
            WaitResult::Child(Ok(exit_status), closed) => {
                if exit_status.success() || closed {
                    log::debug!(
                        "OpenVPN exited, as expected, with exit status: {}",
                        exit_status
                    );
                    Ok(())
                } else {
                    log::error!("OpenVPN died unexpectedly with status: {}", exit_status);
                    Err(self.postmortem())
                }
            }
            WaitResult::Child(Err(e), _) => {
                log::error!("OpenVPN process wait error: {}", e);
                Err(e).chain_err(|| ErrorKind::ChildProcessError("Error when waiting"))
            }
            WaitResult::EventDispatcher => {
                log::error!("OpenVPN Event server exited unexpectedly");
                Err(ErrorKind::EventDispatcherError.into())
            }
        }
    }

    /// Waits for both the child process and the event dispatcher in parallel. After both have
    /// returned this returns the earliest result.
    fn wait_result(&mut self) -> WaitResult {
        let child_wait_handle = self.child.clone();
        let closed_handle = self.closed.clone();
        let child_close_handle = self.close_handle();
        let event_dispatcher = self.event_dispatcher.take().unwrap();
        let dispatcher_handle = event_dispatcher.close_handle();

        let (child_tx, rx) = mpsc::channel();
        let dispatcher_tx = child_tx.clone();

        thread::spawn(move || {
            let result = child_wait_handle.wait();
            let closed = closed_handle.load(Ordering::SeqCst);
            child_tx.send(WaitResult::Child(result, closed)).unwrap();
            dispatcher_handle.close();
        });
        thread::spawn(move || {
            event_dispatcher.wait();
            dispatcher_tx.send(WaitResult::EventDispatcher).unwrap();
            let _ = child_close_handle.close();
        });

        let result = rx.recv().unwrap();
        let _ = rx.recv().unwrap();
        result
    }

    /// Performs a postmortem analysis to attempt to provide a more detailed error result.
    fn postmortem(self) -> Error {
        #[cfg(windows)]
        {
            use std::fs;

            if let Some(log_path) = self.log_path {
                if let Ok(log) = fs::read_to_string(log_path) {
                    if log.contains("There are no TAP-Windows adapters on this system") {
                        return ErrorKind::MissingTapAdapter.into();
                    }
                    if log.contains("CreateFile failed on TAP device") {
                        return ErrorKind::DisabledTapAdapter.into();
                    }
                }
            }
        }

        ErrorKind::ChildProcessError("Died unexpectedly").into()
    }

    fn create_proxy_auth_file(
        proxy: &Option<openvpn::ProxySettings>,
    ) -> ::std::result::Result<Option<mktemp::TempFile>, io::Error> {
        if let Some(openvpn::ProxySettings::Remote(ref remote_proxy)) = proxy {
            if let Some(ref proxy_auth) = remote_proxy.auth {
                return Ok(Some(Self::create_credentials_file(
                    &proxy_auth.username,
                    &proxy_auth.password,
                )?));
            }
        }
        Ok(None)
    }

    fn create_credentials_file(username: &str, password: &str) -> io::Result<mktemp::TempFile> {
        let temp_file = mktemp::TempFile::new();
        log::debug!("Writing credentials to {}", temp_file.as_ref().display());
        let mut file = fs::File::create(&temp_file)?;
        Self::set_user_pass_file_permissions(&file)?;
        write!(file, "{}\n{}\n", username, password)?;
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
            bail!(ErrorKind::PluginNotFound(path));
        }
    }

    fn create_openvpn_cmd(
        params: &openvpn::TunnelParameters,
        tunnel_alias: Option<OsString>,
        user_pass_file: &Path,
        proxy_auth_file: Option<&Path>,
        resource_dir: &Path,
    ) -> Result<OpenVpnCommand> {
        let mut cmd = OpenVpnCommand::new(Self::get_openvpn_bin(resource_dir)?);
        if let Some(config) = Self::get_config_path(resource_dir) {
            cmd.config(config);
        }
        #[cfg(target_os = "linux")]
        cmd.iproute_bin(
            which::which("ip")
                .compat()
                .chain_err(|| ErrorKind::IpRouteNotFound)?,
        );
        cmd.remote(params.config.get_tunnel_endpoint().endpoint)
            .user_pass(user_pass_file)
            .tunnel_options(&params.options)
            .enable_ipv6(params.generic_options.enable_ipv6)
            .tunnel_alias(tunnel_alias)
            .ca(resource_dir.join("ca.crt"));
        if let Some(proxy_auth_file) = proxy_auth_file {
            cmd.proxy_auth(proxy_auth_file);
        }

        Ok(cmd)
    }

    fn get_openvpn_bin(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_BIN_FILENAME);
        if path.exists() {
            log::trace!("Using OpenVPN at {}", path.display());
            Ok(path)
        } else {
            bail!(ErrorKind::OpenVpnNotFound(path));
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
#[derive(Debug, Clone)]
pub struct OpenVpnCloseHandle<H: ProcessHandle = OpenVpnProcHandle> {
    child: Arc<H>,
    closed: Arc<AtomicBool>,
}

impl<H: ProcessHandle> OpenVpnCloseHandle<H> {
    /// Kills the underlying OpenVPN process, making the `OpenVpnMonitor::wait` method return.
    pub fn close(self) -> io::Result<()> {
        if !self.closed.swap(true, Ordering::SeqCst) {
            self.child.kill()
        } else {
            Ok(())
        }
    }
}

/// Internal enum to differentiate between if the child process or the event dispatcher died first.
#[derive(Debug)]
enum WaitResult {
    Child(io::Result<ExitStatus>, bool),
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
}

/// Trait for types acting as handles to subprocesses for `OpenVpnMonitor`
pub trait ProcessHandle: Send + Sync + 'static {
    /// Block until the subprocess exits or there is an error in the wait syscall.
    fn wait(&self) -> io::Result<ExitStatus>;

    /// Kill the subprocess.
    fn kill(&self) -> io::Result<()>;
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
        OpenVpnProcHandle::new(self.build())
    }
}

impl ProcessHandle for OpenVpnProcHandle {
    fn wait(&self) -> io::Result<ExitStatus> {
        self.inner.wait().map(|output| output.status)
    }

    fn kill(&self) -> io::Result<()> {
        self.nice_kill(OPENVPN_DIE_TIMEOUT)
    }
}


mod event_server {
    use jsonrpc_core::{Error, IoHandler, MetaIoHandler};
    use jsonrpc_macros::build_rpc_trait;
    use std::collections::HashMap;
    use talpid_ipc;
    use uuid;

    /// Construct and start the IPC server with the given event listener callback.
    pub fn start<L>(on_event: L) -> talpid_ipc::Result<talpid_ipc::IpcServer>
    where
        L: Fn(openvpn_plugin::EventType, HashMap<String, String>) + Send + Sync + 'static,
    {
        let uuid = uuid::Uuid::new_v4().to_string();
        let ipc_path = if cfg!(windows) {
            format!("//./pipe/talpid-openvpn-{}", uuid)
        } else {
            format!("/tmp/talpid-openvpn-{}", uuid)
        };
        let rpc = OpenVpnEventApiImpl { on_event };
        let mut io = IoHandler::new();
        io.extend_with(rpc.to_delegate());
        let meta_io: MetaIoHandler<()> = MetaIoHandler::from(io);
        talpid_ipc::IpcServer::start(meta_io, &ipc_path)
    }

    build_rpc_trait! {
        pub trait OpenVpnEventApi {

            #[rpc(name = "openvpn_event")]
            fn openvpn_event(&self, openvpn_plugin::EventType, HashMap<String, String>)
                -> Result<(), Error>;
        }
    }

    struct OpenVpnEventApiImpl<L> {
        on_event: L,
    }

    impl<L> OpenVpnEventApi for OpenVpnEventApiImpl<L>
    where
        L: Fn(openvpn_plugin::EventType, HashMap<String, String>) + Send + Sync + 'static,
    {
        fn openvpn_event(
            &self,
            event: openvpn_plugin::EventType,
            env: HashMap<String, String>,
        ) -> Result<(), Error> {
            log::trace!("OpenVPN event {:?}", event);
            (self.on_event)(event, env);
            Ok(())
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

        fn start(&self) -> io::Result<Self::ProcessHandle> {
            self.process_handle
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to start"))
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct TestProcessHandle(i32);

    impl ProcessHandle for TestProcessHandle {
        #[cfg(unix)]
        fn wait(&self) -> io::Result<ExitStatus> {
            use std::os::unix::process::ExitStatusExt;
            Ok(ExitStatus::from_raw(self.0))
        }

        #[cfg(windows)]
        fn wait(&self) -> io::Result<ExitStatus> {
            use std::os::windows::process::ExitStatusExt;
            Ok(ExitStatus::from_raw(self.0 as u32))
        }

        fn kill(&self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn sets_plugin() {
        let builder = TestOpenVpnBuilder::default();
        let _ = OpenVpnMonitor::new_internal(
            builder.clone(),
            |_, _| {},
            "./my_test_plugin",
            None,
            TempFile::new(),
            None,
        );
        assert_eq!(
            Some(PathBuf::from("./my_test_plugin")),
            *builder.plugin.lock().unwrap()
        );
    }

    #[test]
    fn sets_log() {
        let builder = TestOpenVpnBuilder::default();
        let _ = OpenVpnMonitor::new_internal(
            builder.clone(),
            |_, _| {},
            "",
            Some(PathBuf::from("./my_test_log_file")),
            TempFile::new(),
            None,
        );
        assert_eq!(
            Some(PathBuf::from("./my_test_log_file")),
            *builder.log.lock().unwrap()
        );
    }

    #[test]
    fn exit_successfully() {
        let mut builder = TestOpenVpnBuilder::default();
        builder.process_handle = Some(TestProcessHandle(0));
        let testee =
            OpenVpnMonitor::new_internal(builder, |_, _| {}, "", None, TempFile::new(), None)
                .unwrap();
        assert!(testee.wait().is_ok());
    }

    #[test]
    fn exit_error() {
        let mut builder = TestOpenVpnBuilder::default();
        builder.process_handle = Some(TestProcessHandle(1));
        let testee =
            OpenVpnMonitor::new_internal(builder, |_, _| {}, "", None, TempFile::new(), None)
                .unwrap();
        assert!(testee.wait().is_err());
    }

    #[test]
    fn wait_closed() {
        let mut builder = TestOpenVpnBuilder::default();
        builder.process_handle = Some(TestProcessHandle(1));
        let testee =
            OpenVpnMonitor::new_internal(builder, |_, _| {}, "", None, TempFile::new(), None)
                .unwrap();
        testee.close_handle().close().unwrap();
        assert!(testee.wait().is_ok());
    }

    #[test]
    fn failed_process_start() {
        let builder = TestOpenVpnBuilder::default();
        let error =
            OpenVpnMonitor::new_internal(builder, |_, _| {}, "", None, TempFile::new(), None)
                .unwrap_err();
        match error.kind() {
            ErrorKind::ChildProcessError(_) => (),
            _ => panic!("Wrong error"),
        }
    }
}
