use jsonrpc_core::{Error, IoHandler};
use openvpn_ffi;
use process::monitor::ChildMonitor;
use process::openvpn::OpenVpnCommand;
use std::io;

use std::path::{Path, PathBuf};
use std::process;
use std::result::Result as StdResult;
use std::sync::{Arc, Mutex};

use talpid_ipc;

mod errors {
    error_chain!{
        errors {
            /// The `OpenVpnMonitor` is in an invalid state for the requested operation.
            InvalidState {
                description("Invalid state. OpenVPN is already running")
            }
            /// Unable to start or kill the OpenVPN process.
            ChildProcessError {
                description("Unable to start or kill the OpenVPN process")
            }
            /// Unable to start or manage the IPC server listening for events from OpenVPN
            IpcServerError {
                description("Unable to start or manage the IPC server")
            }
        }
    }
}
pub use self::errors::*;


/// Possible events from OpenVPN
#[derive(Debug)]
pub enum OpenVpnEvent {
    /// An event from the plugin loaded into OpenVPN.
    PluginEvent(openvpn_ffi::OpenVpnPluginEvent, openvpn_ffi::OpenVpnEnv),
    /// The OpenVPN process exited. Containing the result of waiting for the process.
    Shutdown(io::Result<process::ExitStatus>),
}

/// Struct for monitoring OpenVPN processes.
pub struct OpenVpnMonitor {
    on_event: Arc<Fn(OpenVpnEvent) + Send + Sync + 'static>,
    plugin_path: PathBuf,
    child: Arc<Mutex<Option<ChildMonitor>>>,
    event_dispatcher: OpenVpnEventDispatcher,
}

impl OpenVpnMonitor {
    /// Creates a new `OpenVpnMonitor` with the given listener and using the plugin at the given
    /// path.
    pub fn new<L, P>(on_event: L, plugin_path: P) -> Result<Self>
        where L: Fn(OpenVpnEvent) + Send + Sync + 'static,
              P: AsRef<Path>
    {
        let on_event = Arc::new(on_event);
        let event_dispatcher = Self::start_event_dispatcher(on_event.clone())?;
        Ok(
            OpenVpnMonitor {
                on_event,
                plugin_path: plugin_path.as_ref().to_owned(),
                child: Arc::new(Mutex::new(None)),
                event_dispatcher,
            },
        )
    }

    fn start_event_dispatcher(on_event: Arc<Fn(OpenVpnEvent) + Send + Sync + 'static>)
                              -> Result<OpenVpnEventDispatcher> {
        let on_plugin_event = move |event, env| (*on_event)(OpenVpnEvent::PluginEvent(event, env));
        OpenVpnEventDispatcher::start(on_plugin_event).chain_err(|| ErrorKind::IpcServerError)
    }

    /// Tries to start a new OpenVPN process if one is not already running.
    /// If this `OpenVpnMonitor is already monitoring a running process it will return an
    /// `InvalidState` error.
    pub fn start(&self, cmd: OpenVpnCommand) -> Result<()> {
        let mut child_lock = self.child.lock().unwrap();
        if child_lock.is_some() {
            bail!(ErrorKind::InvalidState);
        }
        *child_lock = Some(self.start_child_monitor(cmd)?);
        Ok(())
    }

    fn start_child_monitor(&self, mut cmd: OpenVpnCommand) -> Result<ChildMonitor> {
        self.set_plugin(&mut cmd);

        let child = self.child.clone();
        let on_event = self.on_event.clone();

        let on_exit = move |exit_status: io::Result<&process::Output>| {
            *child.lock().unwrap() = None;
            (*on_event)(OpenVpnEvent::Shutdown(exit_status.map(|output| output.status)),)
        };
        ChildMonitor::start(&cmd.build(), on_exit).chain_err(|| ErrorKind::ChildProcessError)
    }

    fn set_plugin(&self, cmd: &mut OpenVpnCommand) {
        let event_dispatcher_address = self.event_dispatcher.address().to_string();
        cmd.plugin(&self.plugin_path, vec![event_dispatcher_address]);
    }

    /// Tries to kill the OpenVPN process if it is running. If it is already dead, this does
    /// nothing.
    pub fn kill(&self) -> Result<()> {
        if let Some(ref child) = *self.child.lock().unwrap() {
            child.kill().chain_err(|| ErrorKind::ChildProcessError)?;
        }
        Ok(())
    }
}




/// IPC server for listening to events coming from plugin loaded into OpenVPN.
pub struct OpenVpnEventDispatcher {
    server: talpid_ipc::IpcServer,
}

impl OpenVpnEventDispatcher {
    /// Construct and start the IPC server with the given event listener callback.
    pub fn start<L>(on_event: L) -> talpid_ipc::Result<Self>
        where L: Fn(openvpn_ffi::OpenVpnPluginEvent, openvpn_ffi::OpenVpnEnv),
              L: Send + Sync + 'static
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

    /// Consumes the server and waits for it to finish.
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
                             openvpn_ffi::OpenVpnPluginEvent,
                             openvpn_ffi::OpenVpnEnv)
                             -> StdResult<(), Error>;
        }
    }
}
use self::api::*;

struct OpenVpnEventApiImpl<L>
    where L: Fn(openvpn_ffi::OpenVpnPluginEvent, openvpn_ffi::OpenVpnEnv) + Send + Sync + 'static
{
    on_event: L,
}

impl<L> OpenVpnEventApi for OpenVpnEventApiImpl<L>
    where L: Fn(openvpn_ffi::OpenVpnPluginEvent, openvpn_ffi::OpenVpnEnv) + Send + Sync + 'static
{
    fn openvpn_event(&self,
                     event: openvpn_ffi::OpenVpnPluginEvent,
                     env: openvpn_ffi::OpenVpnEnv)
                     -> StdResult<(), Error> {
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
