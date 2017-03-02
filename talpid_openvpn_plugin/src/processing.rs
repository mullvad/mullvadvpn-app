

use ffi::OpenVpnPluginEvent;
use std::collections::HashMap;


error_chain!{}


/// Struct processing OpenVPN events and notifies listeners over IPC
pub struct EventProcessor;

impl EventProcessor {
    pub fn new() -> Result<EventProcessor> {
        debug!("Creating EventProcessor");
        Ok(EventProcessor)
    }

    pub fn process_event(&mut self, event: OpenVpnPluginEvent, _env: HashMap<String, String>) {
        // TODO(linus): This is where we should send events to core.
        trace!("Hello from EventProcessor: {:?}", event);
    }
}

impl Drop for EventProcessor {
    fn drop(&mut self) {
        // TODO(linus): If we need, this is where we send some shutdown event or similar to core.
        debug!("Dropping EventProcessor");
    }
}
