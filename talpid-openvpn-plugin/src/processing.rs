use super::Arguments;
use jsonrpc_client_core::{
    expand_params, jsonrpc_client, Future, Result as ClientResult, Transport,
};
use jsonrpc_client_ipc::IpcTransport;
use std::{collections::HashMap, sync::mpsc, thread};
use tokio::{reactor::Handle, runtime::Runtime};

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
    client_result_rx: mpsc::Receiver<ClientResult<()>>,
}

impl EventProcessor {
    pub fn new(arguments: Arguments) -> Result<EventProcessor> {
        log::trace!("Creating EventProcessor");
        let (start_tx, start_rx) = mpsc::channel();
        let (client_result_tx, client_result_rx) = mpsc::channel();
        thread::spawn(move || {
            let mut rt = Runtime::new().expect("failed to spawn runtime");

            let (client, client_handle) =
                IpcTransport::new(&arguments.ipc_socket_path, &Handle::current())
                    .expect("Unable to create IPC transport")
                    .into_client();

            let _ = start_tx.send(client_handle);
            let _ = client_result_tx.send(rt.block_on(client));
        });

        let client_handle = start_rx.recv().chain_err(|| ErrorKind::Shutdown)?;
        let ipc_client = EventProxy::new(client_handle);

        Ok(EventProcessor {
            ipc_client,
            client_result_rx,
        })
    }

    pub fn process_event(
        &mut self,
        event: openvpn_plugin::EventType,
        env: HashMap<String, String>,
    ) -> Result<()> {
        log::trace!("Processing \"{:?}\" event", event);
        let call_future = self
            .ipc_client
            .openvpn_event(event, env)
            .map_err(|e| Error::with_chain(e, ErrorKind::IpcSendingError));
        call_future.wait()?;
        self.check_client_status()
    }

    fn check_client_status(&mut self) -> Result<()> {
        use std::sync::mpsc::TryRecvError::*;
        match self.client_result_rx.try_recv() {
            Err(Empty) => Ok(()),
            Err(Disconnected) | Ok(Ok(())) => Err(ErrorKind::Shutdown.into()),
            Ok(Err(e)) => Err(Error::with_chain(e, ErrorKind::IpcSendingError)),
        }
    }
}

jsonrpc_client!(pub struct EventProxy {
    pub fn openvpn_event(&mut self, event: openvpn_plugin::EventType, env: HashMap<String, String>) -> Future<()>;
});
