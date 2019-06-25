use crate::{get_class, into_java::IntoJava};
use jni::{
    objects::{GlobalRef, JMethodID, JObject, JValue},
    signature::{JavaType, Primitive},
    AttachGuard, JNIEnv,
};
use mullvad_daemon::EventListener;
use mullvad_types::{relay_list::RelayList, settings::Settings, wireguard::KeygenEvent};
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

enum Event {
    RelayList(RelayList),
    Settings(Settings),
    Tunnel(TunnelStateTransition),
}

#[derive(Clone, Debug)]
pub struct JniEventListener(mpsc::Sender<Event>);

impl JniEventListener {
    pub fn spawn(env: &JNIEnv, mullvad_daemon: &JObject) -> Result<Self, Error> {
        JniEventHandler::spawn(env, mullvad_daemon)
    }
}

impl EventListener for JniEventListener {
    fn notify_new_state(&self, state: TunnelStateTransition) {
        let _ = self.0.send(Event::Tunnel(state));
    }

    fn notify_settings(&self, settings: Settings) {
        let _ = self.0.send(Event::Settings(settings));
    }

    fn notify_relay_list(&self, relay_list: RelayList) {
        let _ = self.0.send(Event::RelayList(relay_list));
    }

    // TODO: manage key events properly
    fn notify_key_event(&self, _key_event: KeygenEvent) {}
}

struct JniEventHandler<'env> {
    env: AttachGuard<'env>,
    mullvad_ipc_client: JObject<'env>,
    notify_relay_list_event: JMethodID<'env>,
    notify_settings_event: JMethodID<'env>,
    notify_tunnel_event: JMethodID<'env>,
    events: mpsc::Receiver<Event>,
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
        events: mpsc::Receiver<Event>,
    ) -> Result<Self, Error> {
        let class = get_class("net/mullvad/mullvadvpn/MullvadDaemon");
        let notify_relay_list_event = Self::get_method_id(
            &env,
            &class,
            "notifyRelayListEvent",
            "(Lnet/mullvad/mullvadvpn/model/RelayList;)V",
        )?;
        let notify_settings_event = Self::get_method_id(
            &env,
            &class,
            "notifySettingsEvent",
            "(Lnet/mullvad/mullvadvpn/model/Settings;)V",
        )?;
        let notify_tunnel_event = Self::get_method_id(
            &env,
            &class,
            "notifyTunnelStateEvent",
            "(Lnet/mullvad/mullvadvpn/model/TunnelStateTransition;)V",
        )?;

        Ok(JniEventHandler {
            env,
            mullvad_ipc_client,
            notify_relay_list_event,
            notify_settings_event,
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
            match event {
                Event::RelayList(relay_list) => self.handle_relay_list_event(relay_list),
                Event::Settings(settings) => self.handle_settings(settings),
                Event::Tunnel(tunnel_event) => self.handle_tunnel_event(tunnel_event),
            }
        }
    }

    fn handle_relay_list_event(&self, relay_list: RelayList) {
        let java_relay_list = self.env.auto_local(relay_list.into_java(&self.env));

        let result = self.env.call_method_unchecked(
            self.mullvad_ipc_client,
            self.notify_relay_list_event,
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(java_relay_list.as_obj())],
        );

        if let Err(error) = result {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to call MullvadDaemon.notifyRelayListEvent")
            );
        }
    }

    fn handle_settings(&self, settings: Settings) {
        let java_settings = self.env.auto_local(settings.into_java(&self.env));

        let result = self.env.call_method_unchecked(
            self.mullvad_ipc_client,
            self.notify_settings_event,
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(java_settings.as_obj())],
        );

        if let Err(error) = result {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to call MullvadDaemon.notifySettingsEvent")
            );
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
