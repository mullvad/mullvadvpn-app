use openvpn_plugin;
use std::collections::HashMap;

extern crate futures;

use jsonrpc_client_core::{Future, Result as ClientResult, Transport};
use jsonrpc_client_ipc::IpcTransport;
use tokio_core::reactor::Core;

use super::Arguments;

error_chain! {
    errors {
        IpcSendingError {
            description("Failed while sending an event over the IPC channel")
        }

        Shutdown {
            description("Connection is shut down")
        }

    }
}


/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor {
    ipc_client: EventProxy,
    client_stop: ::std::sync::mpsc::Receiver<ClientResult<()>>,
    core: Core,
}

impl EventProcessor {
    pub fn new(arguments: Arguments) -> Result<EventProcessor> {
        trace!("Creating EventProcessor");
        let core = Core::new().chain_err(|| "Unable to initialize Tokio Core")?;
        let handle = core.handle();
        let (client, client_handle) = IpcTransport::new(&arguments.ipc_socket_path, &handle)
            .chain_err(|| "Unable to create IPC transport")?
            .into_client();

        let (tx, client_stop) = ::std::sync::mpsc::channel();

        let client_future = client.then(move |result| tx.send(result)).map_err(|_| ());
        handle.spawn(client_future);

        let ipc_client = EventProxy::new(client_handle);

        Ok(EventProcessor {
            ipc_client,
            client_stop,
            core,
        })
    }

    pub fn process_event(
        &mut self,
        event: openvpn_plugin::types::OpenVpnPluginEvent,
        env: HashMap<String, String>,
    ) -> Result<()> {
        trace!("Processing \"{:?}\" event", event);
        let call_future = self
            .ipc_client
            .openvpn_event(event, env)
            .map_err(|e| Error::with_chain(e, ErrorKind::IpcSendingError));
        self.core.run(call_future)?;
        self.check_client_status()
    }

    fn check_client_status(&mut self) -> Result<()> {
        use std::sync::mpsc::TryRecvError::*;
        match self.client_stop.try_recv() {
            Err(Empty) => Ok(()),
            Err(Disconnected) => Err(ErrorKind::Shutdown.into()),
            Ok(Ok(_)) => Err(ErrorKind::Shutdown.into()),
            Ok(Err(e)) => Err(Error::with_chain(e, ErrorKind::IpcSendingError)),
        }
    }
}

jsonrpc_client!(pub struct EventProxy {
    pub fn openvpn_event(&mut self, event: openvpn_plugin::types::OpenVpnPluginEvent, env: HashMap<String, String>) -> Future<()>;
});
