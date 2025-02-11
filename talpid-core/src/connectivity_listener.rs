//! Rust wrapper around Android connectivity listener

use futures::channel::mpsc::UnboundedSender;
use jnix::{
    jni::{
        self,
        objects::{GlobalRef, JObject, JValue},
        sys::{jboolean, JNI_TRUE},
        JNIEnv, JavaVM,
    },
    FromJava, JnixEnv,
};
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};
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
    android_listener: GlobalRef,
}

static CONNECTIVITY_TX: Mutex<Option<UnboundedSender<Connectivity>>> = Mutex::new(None);

impl ConnectivityListener {
    /// Create a new connectivity listener
    pub fn new(android_context: AndroidContext) -> Result<Self, Error> {
        let env = JnixEnv::from(
            android_context
                .jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );

        let result = env
            .call_method(
                android_context.vpn_service.as_obj(),
                "getConnectivityListener",
                "()Lnet/mullvad/talpid/ConnectivityListener;",
                &[],
            )
            .map_err(|cause| {
                Error::CallMethod("MullvadVpnService", "getConnectivityListener", cause)
            })?;

        let android_listener = match result {
            JValue::Object(object) => env.new_global_ref(object).map_err(Error::CreateGlobalRef)?,
            value => {
                return Err(Error::InvalidMethodResult(
                    "MullvadVpnService",
                    "getConnectivityListener",
                    format!("{:?}", value),
                ))
            }
        };

        Ok(ConnectivityListener {
            jvm: android_context.jvm,
            android_listener,
        })
    }

    /// Register a channel that receives changes about the offline state.
    ///
    /// # Note
    ///
    /// The listener is shared by all instances of the struct.
    pub fn set_connectivity_listener(&mut self, sender: UnboundedSender<Connectivity>) {
        *CONNECTIVITY_TX.lock().unwrap() = Some(sender);
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
        let env = JnixEnv::from(
            self.jvm
                .attach_current_thread_as_daemon()
                .map_err(Error::AttachJvmToThread)?,
        );

        let is_connected =
            env.call_method(self.android_listener.as_obj(), "isConnected", "()Z", &[]);

        match is_connected {
            Ok(JValue::Bool(JNI_TRUE)) => Ok(true),
            Ok(JValue::Bool(_)) => Ok(false),
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

        let current_dns_servers = env.call_method(
            self.android_listener.as_obj(),
            "getCurrentDnsServers",
            "()Ljava/util/ArrayList;",
            &[],
        );

        match current_dns_servers {
            Ok(JValue::Object(jaddrs)) => Ok(Vec::from_java(&env, jaddrs)),
            value => Err(Error::InvalidMethodResult(
                "ConnectivityListener",
                "getCurrentDnsServers",
                format!("{:?}", value),
            )),
        }
    }
}

/// Entry point for Android Java code to notify the connectivity status.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_notifyConnectivityChange(
    _: JNIEnv<'_>,
    _: JObject<'_>,
    connected: jboolean,
) {
    let Some(tx) = &*CONNECTIVITY_TX.lock().unwrap() else {
        // No sender has been registered
        log::trace!("Received connectivity notification wíth no channel");
        return;
    };

    let connected = JNI_TRUE == connected;

    if tx
        .unbounded_send(Connectivity::Status { connected })
        .is_err()
    {
        log::warn!("Failed to send offline change event");
    }
}
