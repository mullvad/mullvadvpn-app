use crate::{get_class, into_java::IntoJava};
use jni::{
    objects::{GlobalRef, JMethodID, JObject, JValue},
    signature::{JavaType, Primitive},
    AttachGuard, JNIEnv,
};
use mullvad_daemon::EventListener;
use mullvad_types::{relay_list::RelayList, settings::Settings};
use std::{sync::mpsc, thread};
use talpid_types::{tunnel::TunnelStateTransition, ErrorExt};

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Failed to create global reference to MullvadDaemon Java object")]
    CreateGlobalReference(#[error(cause)] jni::errors::Error),

    #[error(display = "Failed to find {} method", _0)]
    FindMethod(&'static str, #[error(cause)] jni::errors::Error),

    #[error(display = "Failed to retrieve Java VM instance")]
    GetJvmInstance(#[error(cause)] jni::errors::Error),
}

#[derive(Clone, Debug)]
pub struct JniEventListener(mpsc::Sender<TunnelStateTransition>);

impl JniEventListener {
    pub fn spawn(env: &JNIEnv, mullvad_daemon: &JObject) -> Result<Self, Error> {
        JniEventHandler::spawn(env, mullvad_daemon)
    }
}

impl EventListener for JniEventListener {
    fn notify_new_state(&self, state: TunnelStateTransition) {
        let _ = self.0.send(state);
    }

    fn notify_settings(&self, _: Settings) {}
    fn notify_relay_list(&self, _: RelayList) {}
}

struct JniEventHandler<'env> {
    env: AttachGuard<'env>,
    mullvad_ipc_client: JObject<'env>,
    notify_tunnel_event: JMethodID<'env>,
    events: mpsc::Receiver<TunnelStateTransition>,
}

impl JniEventHandler<'_> {
    pub fn spawn(
        old_env: &JNIEnv,
        old_mullvad_ipc_client: &JObject,
    ) -> Result<JniEventListener, Error> {
        let (tx, rx) = mpsc::channel();
        let jvm = old_env.get_java_vm().map_err(Error::GetJvmInstance)?;
        let mullvad_ipc_client = old_env
            .new_global_ref(*old_mullvad_ipc_client)
            .map_err(Error::CreateGlobalReference)?;

        thread::spawn(move || match jvm.attach_current_thread() {
            Ok(env) => match JniEventHandler::new(env, mullvad_ipc_client.as_obj(), rx) {
                Ok(mut listener) => listener.run(),
                Err(error) => log::error!("{}", error.display_chain()),
            },
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to attach tunnel event listener thread to Java VM"
                    )
                );
            }
        });

        Ok(JniEventListener(tx))
    }
}

impl<'env> JniEventHandler<'env> {
    fn new(
        env: AttachGuard<'env>,
        mullvad_ipc_client: JObject<'env>,
        events: mpsc::Receiver<TunnelStateTransition>,
    ) -> Result<Self, Error> {
        let class = get_class("net/mullvad/mullvadvpn/MullvadDaemon");
        let notify_tunnel_event = Self::get_method_id(
            &env,
            &class,
            "notifyTunnelStateEvent",
            "(Lnet/mullvad/mullvadvpn/model/TunnelStateTransition;)V",
        )?;

        Ok(JniEventHandler {
            env,
            mullvad_ipc_client,
            notify_tunnel_event,
            events,
        })
    }

    fn get_method_id(
        env: &AttachGuard<'env>,
        class: &GlobalRef,
        method: &'static str,
        signature: &str,
    ) -> Result<JMethodID<'env>, Error> {
        env.get_method_id(class, method, signature)
            .map_err(|error| Error::FindMethod(method, error))
    }

    fn run(&mut self) {
        while let Ok(event) = self.events.recv() {
            self.handle_tunnel_event(event);
        }
    }

    fn handle_tunnel_event(&self, event: TunnelStateTransition) {
        let result = self.env.call_method_unchecked(
            self.mullvad_ipc_client,
            self.notify_tunnel_event,
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(event.into_java(&self.env))],
        );

        if let Err(error) = result {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to call MullvadDaemon.notifyTunnelStateEvent")
            );
        }
    }
}
