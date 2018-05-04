use openvpn_plugin;
use std::collections::HashMap;
use std::sync::Mutex;
use talpid_ipc::{IpcServerId, WsIpcClient};

error_chain! {
    errors {
        IpcSendingError {
            description("Failed while sending an event over the IPC channel")
        }
    }
}


/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor {
    ipc_client: Mutex<WsIpcClient>,
}

impl EventProcessor {
    pub fn new(server_id: &IpcServerId) -> Result<EventProcessor> {
        trace!("Creating EventProcessor");
        let ipc_client =
            WsIpcClient::connect(server_id).chain_err(|| "Unable to create IPC client")?;
        Ok(EventProcessor {
            ipc_client: Mutex::new(ipc_client),
        })
    }

    pub fn process_event(
        &mut self,
        event: openvpn_plugin::types::OpenVpnPluginEvent,
        env: HashMap<String, String>,
    ) -> Result<()> {
        trace!("Processing \"{:?}\" event", event);
        self.ipc_client
            .lock()
            .expect("a thread panicked while using the RPC client in the OpenVPN plugin")
            .call("openvpn_event", &(event, env))
            .map(|_: Option<()>| ())
            .chain_err(|| ErrorKind::IpcSendingError)
    }
}
