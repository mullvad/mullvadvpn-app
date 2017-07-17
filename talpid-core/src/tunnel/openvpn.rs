use duct;
use jsonrpc_core::{Error, IoHandler};
use openvpn_ffi::{OpenVpnEnv, OpenVpnPluginEvent};
use process::openvpn::OpenVpnCommand;

use std::io;
use std::path::Path;
use std::result::Result as StdResult;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use talpid_ipc;

mod errors {
    error_chain!{
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


lazy_static!{
    static ref OPENVPN_DIE_TIMEOUT: Duration = Duration::from_secs(2);
}


/// Struct for monitoring an OpenVPN process.
pub struct OpenVpnMonitor {
    child: Arc<duct::Handle>,
    event_dispatcher: Option<OpenVpnEventDispatcher>,
    closed: Arc<AtomicBool>,
}

impl OpenVpnMonitor {
    /// Creates a new `OpenVpnMonitor` with the given listener and using the plugin at the given
    /// path.
    pub fn new<L, P>(mut cmd: OpenVpnCommand, on_event: L, plugin_path: P) -> Result<Self>
        where L: Fn(OpenVpnPluginEvent, OpenVpnEnv) + Send + Sync + 'static,
              P: AsRef<Path>
    {
        let event_dispatcher = OpenVpnEventDispatcher::start(on_event)
            .chain_err(|| ErrorKind::EventDispatcherError)?;

        cmd.plugin(plugin_path, vec![event_dispatcher.address().to_owned()]);
        let child = cmd.build()
            .start()
            .chain_err(|| ErrorKind::ChildProcessError("Failed to start"))?;

        Ok(
            OpenVpnMonitor {
                child: Arc::new(child),
                event_dispatcher: Some(event_dispatcher),
                closed: Arc::new(AtomicBool::new(false)),
            },
        )
    }

    /// Creates a handle to this monitor, allowing the tunnel to be closed while some other thread
    /// is blocked in `wait`.
    pub fn close_handle(&self) -> OpenVpnCloseHandle {
        OpenVpnCloseHandle {
            child: self.child.clone(),
            closed: self.closed.clone(),
        }
    }

    /// Consumes the monitor and blocks until OpenVPN exits or there is an error in either waiting
    /// for the process or in the event dispatcher.
    pub fn wait(mut self) -> Result<()> {
        match self.wait_result() {
            WaitResult::Child(Ok(exit_status)) => {
                if exit_status.success() || self.closed.load(Ordering::SeqCst) {
                    debug!(
                        "OpenVPN exited, as expected, with exit status: {}",
                        exit_status
                    );
                    Ok(())
                } else {
                    error!("OpenVPN died unexpectedly with status: {}", exit_status);
                    Err(ErrorKind::ChildProcessError("Died unexpectedly").into())
                }
            }
            WaitResult::Child(Err(e)) => {
                error!("OpenVPN process wait error: {}", e);
                Err(e).chain_err(|| ErrorKind::ChildProcessError("Error when waiting"))
            }
            WaitResult::EventDispatcher(result) => {
                error!("OpenVpnEventDispatcher exited unexpectedly: {:?}", result);
                match result {
                    Ok(()) => Err(ErrorKind::EventDispatcherError.into()),
                    Err(e) => Err(e).chain_err(|| ErrorKind::EventDispatcherError),
                }
            }
        }
    }

    /// Waits for both the child process and the event dispatcher in parallel. After both have
    /// returned this returns the earliest result.
    fn wait_result(&mut self) -> WaitResult {
        let child_wait_handle = self.child.clone();
        let child_close_handle = self.close_handle();
        let event_dispatcher = self.event_dispatcher.take().unwrap();
        let dispatcher_handle = event_dispatcher.close_handle();

        let (child_tx, rx) = mpsc::channel();
        let dispatcher_tx = child_tx.clone();

        thread::spawn(
            move || {
                let result = child_wait_handle.wait().map(|output| output.status);
                child_tx.send(WaitResult::Child(result)).unwrap();
                dispatcher_handle.close();
            },
        );
        thread::spawn(
            move || {
                let result = event_dispatcher.wait();
                dispatcher_tx.send(WaitResult::EventDispatcher(result)).unwrap();
                let _ = child_close_handle.close();
            },
        );

        let result = rx.recv().unwrap();
        let _ = rx.recv().unwrap();
        result
    }
}

/// A handle to an `OpenVpnMonitor` for closing it.
pub struct OpenVpnCloseHandle {
    child: Arc<duct::Handle>,
    closed: Arc<AtomicBool>,
}

impl OpenVpnCloseHandle {
    /// Kills the underlying OpenVPN process, making the `OpenVpnMonitor::wait` method return.
    pub fn close(self) -> io::Result<()> {
        if !self.closed.swap(true, Ordering::SeqCst) {
            self.kill_openvpn()
        } else {
            Ok(())
        }
    }

    #[cfg(unix)]
    fn kill_openvpn(self) -> io::Result<()> {
        ::process::unix::nice_kill(self.child, *OPENVPN_DIE_TIMEOUT)
    }

    #[cfg(not(unix))]
    fn kill_openvpn(self) -> io::Result<()> {
        self.child.kill()
    }
}

/// Internal enum to differentiate between if the child process or the event dispatcher died first.
enum WaitResult {
    Child(io::Result<::std::process::ExitStatus>),
    EventDispatcher(talpid_ipc::Result<()>),
}


/// IPC server for listening to events coming from plugin loaded into OpenVPN.
pub struct OpenVpnEventDispatcher {
    server: talpid_ipc::IpcServer,
}

impl OpenVpnEventDispatcher {
    /// Construct and start the IPC server with the given event listener callback.
    pub fn start<L>(on_event: L) -> talpid_ipc::Result<Self>
        where L: Fn(OpenVpnPluginEvent, OpenVpnEnv) + Send + Sync + 'static
    {
        let rpc = OpenVpnEventApiImpl { on_event };
        let mut io = IoHandler::new();
        io.extend_with(rpc.to_delegate());
        let server = talpid_ipc::IpcServer::start(io.into())?;
        Ok(OpenVpnEventDispatcher { server })
    }

    /// Returns the local address this server is listening on.
    pub fn address(&self) -> &str {
        self.server.address()
    }

    /// Creates a handle to this event dispatcher, allowing the listening server to be closed while
    /// some other thread is blocked in `wait`.
    pub fn close_handle(&self) -> talpid_ipc::CloseHandle {
        self.server.close_handle()
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(self) -> talpid_ipc::Result<()> {
        self.server.wait()
    }
}


mod api {
    use super::*;
    build_rpc_trait! {
        pub trait OpenVpnEventApi {
            #[rpc(name = "openvpn_event")]
            fn openvpn_event(&self,
                             OpenVpnPluginEvent,
                             OpenVpnEnv)
                             -> StdResult<(), Error>;
        }
    }
}
use self::api::*;

struct OpenVpnEventApiImpl<L>
    where L: Fn(OpenVpnPluginEvent, OpenVpnEnv) + Send + Sync + 'static
{
    on_event: L,
}

impl<L> OpenVpnEventApi for OpenVpnEventApiImpl<L>
    where L: Fn(OpenVpnPluginEvent, OpenVpnEnv) + Send + Sync + 'static
{
    fn openvpn_event(&self, event: OpenVpnPluginEvent, env: OpenVpnEnv) -> StdResult<(), Error> {
        debug!("OpenVPN event {:?}", event);
        (self.on_event)(event, env);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn openvpn_event_dispatcher_server() {
        let server = OpenVpnEventDispatcher::start(
            |event, env| {
                println!("event: {:?}. env: {:?}", event, env);
            },
        )
                .unwrap();
        println!("plugin server listening on {}", server.address());
        server.wait().unwrap();
    }
}
