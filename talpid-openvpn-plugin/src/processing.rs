use super::{Arguments, Error};
use jsonrpc_client_core::{
    expand_params, jsonrpc_client, Future, Result as ClientResult, Transport,
};
use jsonrpc_client_ipc::IpcTransport;
use std::{collections::HashMap, sync::mpsc, thread};
use tokio::{reactor::Handle, runtime::Runtime};


/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor {
    ipc_client: EventProxy,
    client_result_rx: mpsc::Receiver<ClientResult<()>>,
}

impl EventProcessor {
    pub fn new(arguments: Arguments) -> Result<EventProcessor, Error> {
        log::trace!("Creating EventProcessor");
        let (start_tx, start_rx) = mpsc::channel();
        let (client_result_tx, client_result_rx) = mpsc::channel();
        thread::spawn(move || {
            let mut rt = match Runtime::new().map_err(Error::CreateRuntime) {
                Err(e) => {
                    let _ = start_tx.send(Err(e));
                    return;
                }
                Ok(rt) => rt,
            };
            let (client, client_handle) =
                match IpcTransport::new(&arguments.ipc_socket_path, &Handle::default())
                    .map_err(Error::CreateTransport)
                    .map(|transport| transport.into_client())
                {
                    Err(e) => {
                        let _ = start_tx.send(Err(e));
                        return;
                    }
                    Ok((client, client_handle)) => (client, client_handle),
                };

            let _ = start_tx.send(Ok(client_handle));
            let _ = client_result_tx.send(rt.block_on(client));
        });

        let client_handle = start_rx
            .recv()
            .expect("No start result from EventProcessor thread")?;
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
    ) -> Result<(), Error> {
        log::trace!("Processing \"{:?}\" event", event);
        let call_future = self
            .ipc_client
            .openvpn_event(event, env)
            .map_err(Error::SendEvent);
        call_future.wait()?;
        self.check_client_status()
    }

    fn check_client_status(&mut self) -> Result<(), Error> {
        use std::sync::mpsc::TryRecvError::*;
        match self.client_result_rx.try_recv() {
            Err(Empty) => Ok(()),
            Err(Disconnected) | Ok(Ok(())) => Err(Error::Shutdown),
            Ok(Err(e)) => Err(Error::SendEvent(e)),
        }
    }
}

jsonrpc_client!(pub struct EventProxy {
    pub fn openvpn_event(&mut self, event: openvpn_plugin::EventType, env: HashMap<String, String>) -> Future<()>;
});
