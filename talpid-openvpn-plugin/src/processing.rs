use openvpn_ffi;

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
    ipc_client: WsIpcClient,
}

impl EventProcessor {
    pub fn new(server_id: IpcServerId) -> Result<EventProcessor> {
        debug!("Creating EventProcessor");
        let ipc_client = WsIpcClient::new(server_id).chain_err(|| "Unable to create IPC client")?;
        Ok(EventProcessor { ipc_client })
    }

    pub fn process_event(&mut self,
                         event: openvpn_ffi::OpenVpnPluginEvent,
                         env: openvpn_ffi::OpenVpnEnv)
                         -> Result<()> {
        trace!("Processing \"{:?}\" event", event);
        self.ipc_client
            .call("openvpn_event", &(event, env))
            .map(|_: Option<()>| ())
            .chain_err(|| ErrorKind::IpcSendingError)
    }
}

impl Drop for EventProcessor {
    fn drop(&mut self) {
        // TODO(linus): If we need, this is where we send some shutdown event or similar to core.
        debug!("Dropping EventProcessor");
    }
}
