use jsonrpc_core::{Error, IoHandler};
use openvpn_ffi::{OpenVpnEnv, OpenVpnPluginEvent};

use talpid_ipc;

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
            fn openvpn_event(&self, OpenVpnPluginEvent, OpenVpnEnv) -> Result<(), Error>;
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
    fn openvpn_event(&self, event: OpenVpnPluginEvent, env: OpenVpnEnv) -> Result<(), Error> {
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
