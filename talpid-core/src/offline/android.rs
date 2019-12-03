use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;
use jnix::jni::{objects::JObject, sys::jlong, JNIEnv};
use std::sync::Weak;

#[derive(err_derive::Error, Debug)]
#[error(display = "Unknown offline monitor error")]
pub struct Error;

pub struct MonitorHandle;

impl MonitorHandle {
    pub fn is_offline(&self) -> bool {
        false
    }
}

/// Entry point for Android Java code to return ownership of the sender reference.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_destroySender(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    sender_address: jlong,
) {
    let _ = unsafe { get_sender_from_address(sender_address) };
}

unsafe fn get_sender_from_address(address: jlong) -> Box<Weak<UnboundedSender<TunnelCommand>>> {
    Box::from_raw(address as *mut Weak<UnboundedSender<TunnelCommand>>)
}

pub fn spawn_monitor(
    _sender: Weak<UnboundedSender<TunnelCommand>>,
) -> Result<MonitorHandle, Error> {
    Ok(MonitorHandle)
}
