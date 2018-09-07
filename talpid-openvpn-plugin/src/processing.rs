use openvpn_plugin;
use std::collections::HashMap;

extern crate futures;

use jsonrpc_client_core::{Future, Result as ClientResult, Transport};
use jsonrpc_client_ipc::IpcTransport;

use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use super::Arguments;
use std::thread;

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
}

impl EventProcessor {
    pub fn new(arguments: Arguments) -> Result<EventProcessor> {
        trace!("Creating EventProcessor");
        let (start_tx, start_rx) = futures::sync::oneshot::channel();
        thread::spawn(move || {
            let mut rt = Runtime::new().expect("failed to spawn runtime");

            let (client, client_handle) =
                IpcTransport::new(&arguments.ipc_socket_path, &Handle::current())
                    .expect("Unable to create IPC transport")
                    .into_client();

            let (tx, client_stop) = ::std::sync::mpsc::channel();
            let client_future = client.then(move |result| tx.send(result)).map_err(|_| ());
            start_tx
                .send((client_stop, client_handle))
                .expect("failed to send client handles");

            rt.block_on(client_future)
                .expect("RPC client should not fail");
        });

        let (client_stop, client_handle) = start_rx.wait().chain_err(|| ErrorKind::Shutdown)?;
        let ipc_client = EventProxy::new(client_handle);

        Ok(EventProcessor {
            ipc_client,
            client_stop,
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
        call_future.wait()?;
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
