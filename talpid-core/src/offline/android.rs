use futures::channel::mpsc::UnboundedSender;
use jnix::{
    jni::{
        self,
        objects::{GlobalRef, JObject, JValue},
        signature::{JavaType, Primitive},
        sys::{jboolean, jlong, JNI_FALSE},
        JNIEnv, JavaVM,
    },
    JnixEnv,
};
use std::sync::{Arc, Weak};
use talpid_types::{android::AndroidContext, ErrorExt};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[error(source)] jni::errors::Error),

    #[error(display = "Failed to call Java method {}.{}", _0, _1)]
    CallMethod(
        &'static str,
        &'static str,
        #[error(source)] jni::errors::Error,
    ),

    #[error(display = "Failed to create global reference to Java object")]
    CreateGlobalRef(#[error(source)] jni::errors::Error),

    #[error(display = "Failed to find {}.{} method", _0, _1)]
    FindMethod(
        &'static str,
        &'static str,
        #[error(source)] jni::errors::Error,
    ),

    #[error(display = "Received an invalid result from {}.{}: {}", _0, _1, _2)]
    InvalidMethodResult(&'static str, &'static str, String),
}

pub struct MonitorHandle {
    jvm: Arc<JavaVM>,
    class: GlobalRef,
    object: GlobalRef,
    _sender: Arc<UnboundedSender<bool>>,
}

impl MonitorHandle {
    pub fn new(
        android_context: AndroidContext,
        sender: Arc<UnboundedSender<bool>>,
    ) -> Result<Self, Error> {
        let env = JnixEnv::from(
            android_context
                .jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );

        let get_connectivity_listener_method = env
            .get_method_id(
                &env.get_class("net/mullvad/talpid/TalpidVpnService"),
                "getConnectivityListener",
                "()Lnet/mullvad/talpid/ConnectivityListener;",
            )
            .map_err(|cause| {
                Error::FindMethod("MullvadVpnService", "getConnectivityListener", cause)
            })?;

        let result = env
            .call_method_unchecked(
                android_context.vpn_service.as_obj(),
                get_connectivity_listener_method,
                JavaType::Object("Lnet/mullvad/talpid/ConnectivityListener;".to_owned()),
                &[],
            )
            .map_err(|cause| {
                Error::CallMethod("MullvadVpnService", "getConnectivityListener", cause)
            })?;

        let object = match result {
            JValue::Object(object) => env.new_global_ref(object).map_err(Error::CreateGlobalRef)?,
            value => {
                return Err(Error::InvalidMethodResult(
                    "MullvadVpnService",
                    "getConnectivityListener",
                    format!("{:?}", value),
                ))
            }
        };

        let class = env.get_class("net/mullvad/talpid/ConnectivityListener");

        Ok(MonitorHandle {
            jvm: android_context.jvm,
            class,
            object,
            _sender: sender,
        })
    }

    pub async fn host_is_offline(&self) -> bool {
        match self.get_is_connected() {
            Ok(is_connected) => !is_connected,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to check connectivity status")
                );
                false
            }
        }
    }

    fn get_is_connected(&self) -> Result<bool, Error> {
        let result = self.call_method(
            "isConnected",
            "()Z",
            &[],
            JavaType::Primitive(Primitive::Boolean),
        )?;

        match result {
            JValue::Bool(JNI_FALSE) => Ok(false),
            JValue::Bool(_) => Ok(true),
            value => Err(Error::InvalidMethodResult(
                "ConnectivityListener",
                "isConnected",
                format!("{:?}", value),
            )),
        }
    }

    fn set_sender(&self, sender: Weak<UnboundedSender<bool>>) -> Result<(), Error> {
        let sender_ptr = Box::new(sender);
        let sender_address = Box::into_raw(sender_ptr) as jlong;

        let result = self.call_method(
            "setSenderAddress",
            "(J)V",
            &[JValue::Long(sender_address)],
            JavaType::Primitive(Primitive::Void),
        )?;

        match result {
            JValue::Void => Ok(()),
            value => Err(Error::InvalidMethodResult(
                "ConnectivityListener",
                "setSenderAddress",
                format!("{:?}", value),
            )),
        }
    }

    fn call_method(
        &self,
        method: &'static str,
        signature: &str,
        parameters: &[JValue<'_>],
        return_type: JavaType,
    ) -> Result<JValue<'_>, Error> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );

        let method_id = env
            .get_method_id(&self.class, method, signature)
            .map_err(|cause| Error::FindMethod("ConnectivityListener", method, cause))?;

        env.call_method_unchecked(self.object.as_obj(), method_id, return_type, parameters)
            .map_err(|cause| Error::CallMethod("ConnectivityListener", method, cause))
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
    let is_offline = is_connected == JNI_FALSE;

    if let Some(sender) = sender_ref.upgrade() {
        if sender.unbounded_send(is_offline).is_err() {
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

unsafe fn get_sender_from_address(address: jlong) -> Box<Weak<UnboundedSender<bool>>> {
    Box::from_raw(address as *mut Weak<UnboundedSender<bool>>)
}

pub async fn spawn_monitor(
    sender: UnboundedSender<bool>,
    android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    let sender = Arc::new(sender);
    let weak_sender = Arc::downgrade(&sender);
    let monitor_handle = MonitorHandle::new(android_context, sender)?;

    monitor_handle.set_sender(weak_sender)?;

    Ok(monitor_handle)
}
