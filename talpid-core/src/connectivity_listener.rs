//! Rust wrapper around Android connectivity listener

use futures::channel::mpsc::UnboundedSender;
use jnix::{
    jni::{
        self,
        objects::{GlobalRef, JObject, JValue},
        signature::{JavaType, Primitive},
        sys::{jboolean, jlong, JNI_TRUE},
        JNIEnv, JavaVM,
    },
    FromJava, JnixEnv,
};
use std::{net::IpAddr, sync::Arc};
use talpid_types::{android::AndroidContext, net::Connectivity, ErrorExt};

/// Error related to Android connectivity monitor
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to attach Java VM to tunnel thread
    #[error("Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[source] jni::errors::Error),

    /// Failed to call Java method
    #[error("Failed to call Java method {0}.{1}")]
    CallMethod(&'static str, &'static str, #[source] jni::errors::Error),

    /// Failed to create global reference to Java object
    #[error("Failed to create global reference to Java object")]
    CreateGlobalRef(#[source] jni::errors::Error),

    /// Failed to find method
    #[error("Failed to find {0}.{1} method")]
    FindMethod(&'static str, &'static str, #[source] jni::errors::Error),

    /// Method returned invalid result
    #[error("Received an invalid result from {0}.{1}: {2}")]
    InvalidMethodResult(&'static str, &'static str, String),
}

/// Android connectivity listener
#[derive(Clone)]
pub struct ConnectivityListener {
    jvm: Arc<JavaVM>,
    class: GlobalRef,
    object: GlobalRef,
}

impl ConnectivityListener {
    /// Create a new connectivity listener
    pub fn new(android_context: AndroidContext) -> Result<Self, Error> {
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

        Ok(ConnectivityListener {
            jvm: android_context.jvm,
            class,
            object,
        })
    }

    /// Register a channel that receives changes about the offline state
    ///
    /// # Note
    ///
    /// The listener is shared by all instances of the struct.
    pub fn set_connectivity_listener(
        &mut self,
        sender: UnboundedSender<Connectivity>,
    ) -> Result<(), Error> {
        let sender_ptr = Box::into_raw(Box::new(sender)) as jlong;

        let result = self.call_method(
            "setSenderAddress",
            "(J)V",
            &[JValue::Long(sender_ptr)],
            JavaType::Primitive(Primitive::Void),
        )?;

        match result {
            JValue::Void => Ok(()),
            value => Err(Error::InvalidMethodResult(
                "ConnectivityListener",
                "setSenderAddress",
                format!("{:?}", value),
            )),
        }?;

        Ok(())
    }

    /// Return the current offline/connectivity state
    pub fn connectivity(&self) -> Connectivity {
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

    /// Return the current DNS servers according to Android
    pub fn current_dns_servers(&self) -> Result<Vec<IpAddr>, Error> {
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );

        let current_dns_servers = self.call_method(
            "getCurrentDnsServers",
            "()Ljava/util/ArrayList;",
            &[],
            JavaType::Object("java/util/ArrayList".to_owned()),
        )?;

        match current_dns_servers {
            JValue::Object(jaddrs) => Ok(Vec::from_java(&env, jaddrs)),
            value => Err(Error::InvalidMethodResult(
                "ConnectivityListener",
                "currentDnsServers",
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

    let sender = unsafe { Box::from_raw(sender_address as *mut UnboundedSender<Connectivity>) };

    if sender
        .unbounded_send(Connectivity::Status { connected })
        .is_err()
    {
        log::warn!("Failed to send offline change event");
    }

    // Do not destroy
    std::mem::forget(sender);
}

/// Entry point for Android Java code to return ownership of the sender reference.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_destroySender(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    sender_address: jlong,
) {
    let _ = unsafe { Box::from_raw(sender_address as *mut UnboundedSender<Connectivity>) };
}
