use crate::process::{
    openvpn::{OpenVpnCommand, OpenVpnProcHandle},
    stoppable_process::StoppableProcess,
};


use std::{
    collections::HashMap,
    io,
    path::Path,
    process::ExitStatus,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use mktemp;
use talpid_ipc;

mod errors {
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
        }
    }
}
pub use self::errors::*;


#[cfg(unix)]
static OPENVPN_DIE_TIMEOUT: Duration = Duration::from_secs(4);
#[cfg(windows)]
static OPENVPN_DIE_TIMEOUT: Duration = Duration::from_secs(30);


/// Struct for monitoring an OpenVPN process.
#[derive(Debug)]
pub struct OpenVpnMonitor<C: OpenVpnBuilder = OpenVpnCommand> {
    child: Arc<C::ProcessHandle>,
    event_dispatcher: Option<talpid_ipc::IpcServer>,
    closed: Arc<AtomicBool>,
    /// Keep the `TempFile` for the user-pass file in the struct, so it's removed on drop.
    _user_pass_file: mktemp::TempFile,
    /// Keep the 'TempFile' for the proxy user-pass file in the struct, so it's removed on drop.
    _proxy_auth_file: Option<mktemp::TempFile>,
}

impl OpenVpnMonitor<OpenVpnCommand> {
    /// Creates a new `OpenVpnMonitor` with the given listener and using the plugin at the given
    /// path.
    pub fn start<L, P>(
        cmd: OpenVpnCommand,
        on_event: L,
        plugin_path: P,
        user_pass_file: mktemp::TempFile,
        proxy_auth_file: Option<mktemp::TempFile>,
    ) -> Result<Self>
    where
        L: Fn(openvpn_plugin::EventType, HashMap<String, String>) + Send + Sync + 'static,
        P: AsRef<Path>,
    {
        Self::new_internal(cmd, on_event, plugin_path, user_pass_file, proxy_auth_file)
    }
}

impl<C: OpenVpnBuilder> OpenVpnMonitor<C> {
    fn new_internal<L, P>(
        mut cmd: C,
        on_event: L,
        plugin_path: P,
        user_pass_file: mktemp::TempFile,
        proxy_auth_file: Option<mktemp::TempFile>,
    ) -> Result<OpenVpnMonitor<C>>
    where
        L: Fn(openvpn_plugin::EventType, HashMap<String, String>) + Send + Sync + 'static,
        P: AsRef<Path>,
    {
        let event_dispatcher =
            event_server::start(on_event).chain_err(|| ErrorKind::EventDispatcherError)?;

        cmd.plugin(plugin_path, vec![event_dispatcher.path().to_owned()]);
        let child = cmd
            .start()
            .chain_err(|| ErrorKind::ChildProcessError("Failed to start"))?;

        Ok(OpenVpnMonitor {
            child: Arc::new(child),
            event_dispatcher: Some(event_dispatcher),
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
                    Err(ErrorKind::ChildProcessError("Died unexpectedly").into())
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
    fn plugin<P: AsRef<Path>>(&mut self, path: P, args: Vec<String>) -> &mut Self;

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

    fn plugin<P: AsRef<Path>>(&mut self, path: P, args: Vec<String>) -> &mut Self {
        self.plugin(path, args)
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
    use std::path::{Path, PathBuf};

    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default, Clone)]
    struct TestOpenVpnBuilder {
        pub plugin: Arc<Mutex<Option<PathBuf>>>,
        pub process_handle: Option<TestProcessHandle>,
    }

    impl OpenVpnBuilder for TestOpenVpnBuilder {
        type ProcessHandle = TestProcessHandle;

        fn plugin<P: AsRef<Path>>(&mut self, path: P, _args: Vec<String>) -> &mut Self {
            *self.plugin.lock().unwrap() = Some(path.as_ref().to_path_buf());
            self
        }

        fn start(&self) -> io::Result<Self::ProcessHandle> {
            self.process_handle
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to start"))
        }
    }

    fn tmpfile() -> mktemp::TempFile {
        mktemp::TempFile::new()
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
            tmpfile(),
            None,
        );
        assert_eq!(
            Some(PathBuf::from("./my_test_plugin")),
            *builder.plugin.lock().unwrap()
        );
    }

    #[test]
    fn exit_successfully() {
        let mut builder = TestOpenVpnBuilder::default();
        builder.process_handle = Some(TestProcessHandle(0));
        let testee = OpenVpnMonitor::new_internal(builder, |_, _| {}, "", tmpfile(), None).unwrap();
        assert!(testee.wait().is_ok());
    }

    #[test]
    fn exit_error() {
        let mut builder = TestOpenVpnBuilder::default();
        builder.process_handle = Some(TestProcessHandle(1));
        let testee = OpenVpnMonitor::new_internal(builder, |_, _| {}, "", tmpfile(), None).unwrap();
        assert!(testee.wait().is_err());
    }

    #[test]
    fn wait_closed() {
        let mut builder = TestOpenVpnBuilder::default();
        builder.process_handle = Some(TestProcessHandle(1));
        let testee = OpenVpnMonitor::new_internal(builder, |_, _| {}, "", tmpfile(), None).unwrap();
        testee.close_handle().close().unwrap();
        assert!(testee.wait().is_ok());
    }

    #[test]
    fn failed_process_start() {
        let builder = TestOpenVpnBuilder::default();
        let error =
            OpenVpnMonitor::new_internal(builder, |_, _| {}, "", tmpfile(), None).unwrap_err();
        match error.kind() {
            ErrorKind::ChildProcessError(_) => (),
            _ => panic!("Wrong error"),
        }
    }
}
