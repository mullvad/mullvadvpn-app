use futures::channel::mpsc::UnboundedSender;
use jnix::{
    jni::{
        self,
        objects::{GlobalRef, JObject, JValue},
        signature::{JavaType, Primitive},
        sys::{jboolean, jlong, JNI_TRUE},
        JNIEnv, JavaVM,
    },
    JnixEnv,
};
use std::sync::{Arc, Weak};
use talpid_types::{android::AndroidContext, net::Connectivity, ErrorExt};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[source] jni::errors::Error),

    #[error("Failed to call Java method {0}.{1}")]
    CallMethod(&'static str, &'static str, #[source] jni::errors::Error),

    #[error("Failed to create global reference to Java object")]
    CreateGlobalRef(#[source] jni::errors::Error),

    #[error("Failed to find {0}.{1} method")]
    FindMethod(&'static str, &'static str, #[source] jni::errors::Error),

    #[error("Received an invalid result from {0}.{1}: {2}")]
    InvalidMethodResult(&'static str, &'static str, String),
}

pub struct MonitorHandle {
    jvm: Arc<JavaVM>,
    class: GlobalRef,
    object: GlobalRef,
    _sender: Arc<UnboundedSender<Connectivity>>,
}

impl MonitorHandle {
    pub fn new(
        android_context: AndroidContext,
        sender: Arc<UnboundedSender<Connectivity>>,
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

    #[allow(clippy::unused_async)]
    pub async fn connectivity(&self) -> Connectivity {
        self.get_is_connected()
            .map(|connected| Connectivity::Status { connected })
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to check connectivity status")
                );
                Connectivity::PresumeOnline
            })
    }

    fn get_is_connected(&self) -> Result<bool, Error> {
        let is_connected = self.call_method(
            "isConnected",
            "()Z",
            &[],
            JavaType::Primitive(Primitive::Boolean),
        )?;

        match is_connected {
            JValue::Bool(JNI_TRUE) => Ok(true),
            JValue::Bool(_) => Ok(false),
            value => Err(Error::InvalidMethodResult(
                "ConnectivityListener",
                "isConnected",
                format!("{:?}", value),
            )),
        }
    }

    fn set_sender(&self, sender: Weak<UnboundedSender<Connectivity>>) -> Result<(), Error> {
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
    connected: jboolean,
    sender_address: jlong,
) {
    let connected = JNI_TRUE == connected;
    let sender_ref = Box::leak(unsafe { get_sender_from_address(sender_address) });
    if let Some(sender) = sender_ref.upgrade() {
        if sender
            .unbounded_send(Connectivity::Status { connected })
            .is_err()
        {
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

unsafe fn get_sender_from_address(address: jlong) -> Box<Weak<UnboundedSender<Connectivity>>> {
    Box::from_raw(address as *mut Weak<UnboundedSender<Connectivity>>)
}

#[allow(clippy::unused_async)]
pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    let sender = Arc::new(sender);
    let weak_sender = Arc::downgrade(&sender);
    let monitor_handle = MonitorHandle::new(android_context, sender)?;

    monitor_handle.set_sender(weak_sender)?;

    Ok(monitor_handle)
}
