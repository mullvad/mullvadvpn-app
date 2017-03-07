use ffi::OpenVpnPluginEvent;

use std::collections::HashMap;

use talpid_ipc::{IpcClient, IpcServerId};

error_chain! {
    errors {
        IpcSendingError {
            description("Failed while sending an event over the IPC channel")
        }
    }
}


/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor {
    ipc_client: IpcClient<HashMap<String, String>>,
}

impl EventProcessor {
    pub fn new(server_id: IpcServerId) -> Result<EventProcessor> {
        debug!("Creating EventProcessor");
        let ipc_client = IpcClient::new(server_id);
        Ok(EventProcessor { ipc_client: ipc_client })
    }

    pub fn process_event(&mut self,
                         event: OpenVpnPluginEvent,
                         env: HashMap<String, String>)
                         -> Result<()> {
        trace!("Processing \"{:?}\" event", event);
        self.ipc_client.send(&env).chain_err(|| ErrorKind::IpcSendingError)
    }
}

impl Drop for EventProcessor {
    fn drop(&mut self) {
        // TODO(linus): If we need, this is where we send some shutdown event or similar to core.
        debug!("Dropping EventProcessor");
    }
}
