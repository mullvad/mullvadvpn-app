use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;
use jnix::jni::{
    objects::JObject,
    sys::{jboolean, jlong, JNI_FALSE},
    JNIEnv,
};
use std::sync::Weak;
use talpid_types::android::AndroidContext;

#[derive(err_derive::Error, Debug)]
#[error(display = "Unknown offline monitor error")]
pub struct Error;

pub struct MonitorHandle;

impl MonitorHandle {
    pub fn is_offline(&self) -> bool {
        false
    }
}

/// Entry point for Android Java code to notify the connectivity status.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_notifyConnectivityChange(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    is_connected: jboolean,
    sender_address: jlong,
) {
    let sender_ref = Box::leak(unsafe { get_sender_from_address(sender_address) });
    let tunnel_command = TunnelCommand::IsOffline(is_connected == JNI_FALSE);

    if let Some(sender) = sender_ref.upgrade() {
        if sender.unbounded_send(tunnel_command).is_err() {
            log::warn!("Failed to send offline change event");
        }
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
    _android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    Ok(MonitorHandle)
}
