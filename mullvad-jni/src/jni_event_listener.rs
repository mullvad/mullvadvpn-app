use jnix::{
    jni::{
        objects::{GlobalRef, JMethodID, JObject, JValue},
        signature::{JavaType, Primitive},
    },
    IntoJava, JnixEnv,
};
use mullvad_daemon::EventListener;
use mullvad_types::{
    relay_list::RelayList, settings::Settings, states::TunnelState, version::AppVersionInfo,
    wireguard::KeygenEvent,
};
use std::{sync::mpsc, thread};
use talpid_types::ErrorExt;

#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to create global reference to MullvadDaemon Java object")]
    CreateGlobalReference(#[error(source)] jnix::jni::errors::Error),

    #[error(display = "Failed to find {} method", _0)]
    FindMethod(&'static str, #[error(source)] jnix::jni::errors::Error),

    #[error(display = "Failed to retrieve Java VM instance")]
    GetJvmInstance(#[error(source)] jnix::jni::errors::Error),
}

enum Event {
    KeygenEvent(KeygenEvent),
    RelayList(RelayList),
    Settings(Settings),
    Tunnel(TunnelState),
    AppVersionInfo(AppVersionInfo),
}

#[derive(Clone, Debug)]
pub struct JniEventListener(mpsc::Sender<Event>);

impl JniEventListener {
    pub fn spawn(env: &JnixEnv<'_>, mullvad_daemon: &JObject<'_>) -> Result<Self, Error> {
        JniEventHandler::spawn(env, mullvad_daemon)
    }
}

impl EventListener for JniEventListener {
    fn notify_key_event(&self, key_event: KeygenEvent) {
        let _ = self.0.send(Event::KeygenEvent(key_event));
    }

    fn notify_new_state(&self, state: TunnelState) {
        let _ = self.0.send(Event::Tunnel(state));
    }

    fn notify_settings(&self, settings: Settings) {
        let _ = self.0.send(Event::Settings(settings));
    }

    fn notify_relay_list(&self, relay_list: RelayList) {
        let _ = self.0.send(Event::RelayList(relay_list));
    }

    fn notify_app_version(&self, app_version_info: AppVersionInfo) {
        let _ = self.0.send(Event::AppVersionInfo(app_version_info));
    }
}

struct JniEventHandler<'env> {
    env: JnixEnv<'env>,
    mullvad_ipc_client: JObject<'env>,
    notify_app_version_info_event: JMethodID<'env>,
    notify_keygen_event: JMethodID<'env>,
    notify_relay_list_event: JMethodID<'env>,
    notify_settings_event: JMethodID<'env>,
    notify_tunnel_event: JMethodID<'env>,
    events: mpsc::Receiver<Event>,
}

impl JniEventHandler<'_> {
    pub fn spawn(
        old_env: &JnixEnv<'_>,
        old_mullvad_ipc_client: &JObject<'_>,
    ) -> Result<JniEventListener, Error> {
        let (tx, rx) = mpsc::channel();
        let jvm = old_env.get_java_vm().map_err(Error::GetJvmInstance)?;
        let mullvad_ipc_client = old_env
            .new_global_ref(*old_mullvad_ipc_client)
            .map_err(Error::CreateGlobalReference)?;

        thread::spawn(move || match jvm.attach_current_thread() {
            Ok(attach_guard) => {
                let env = JnixEnv::from(attach_guard.clone());

                match JniEventHandler::new(env, mullvad_ipc_client.as_obj(), rx) {
                    Ok(mut listener) => listener.run(),
                    Err(error) => log::error!("{}", error.display_chain()),
                }
            }
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
        env: JnixEnv<'env>,
        mullvad_ipc_client: JObject<'env>,
        events: mpsc::Receiver<Event>,
    ) -> Result<Self, Error> {
        let class = env.get_class("net/mullvad/mullvadvpn/service/MullvadDaemon");
        let notify_app_version_info_event = Self::get_method_id(
            &env,
            &class,
            "notifyAppVersionInfoEvent",
            "(Lnet/mullvad/mullvadvpn/model/AppVersionInfo;)V",
        )?;
        let notify_keygen_event = Self::get_method_id(
            &env,
            &class,
            "notifyKeygenEvent",
            "(Lnet/mullvad/mullvadvpn/model/KeygenEvent;)V",
        )?;
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
            "(Lnet/mullvad/mullvadvpn/model/TunnelState;)V",
        )?;

        Ok(JniEventHandler {
            env,
            mullvad_ipc_client,
            notify_app_version_info_event,
            notify_keygen_event,
            notify_relay_list_event,
            notify_settings_event,
            notify_tunnel_event,
            events,
        })
    }

    fn get_method_id(
        env: &JnixEnv<'env>,
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
                Event::KeygenEvent(keygen_event) => self.handle_keygen_event(keygen_event),
                Event::RelayList(relay_list) => self.handle_relay_list_event(relay_list),
                Event::Settings(settings) => self.handle_settings(settings),
                Event::Tunnel(tunnel_event) => self.handle_tunnel_event(tunnel_event),
                Event::AppVersionInfo(app_version_info) => {
                    self.handle_app_version_info_event(app_version_info)
                }
            }
        }
    }

    fn handle_keygen_event(&self, event: KeygenEvent) {
        let java_keygen_event = event.into_java(&self.env);

        let result = self.env.call_method_unchecked(
            self.mullvad_ipc_client,
            self.notify_keygen_event,
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(java_keygen_event.as_obj())],
        );

        if let Err(error) = result {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to call MullvadDaemon.notifyKeygenEvent")
            );
        }
    }

    fn handle_relay_list_event(&self, relay_list: RelayList) {
        let java_relay_list = relay_list.into_java(&self.env);

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
        let java_settings = settings.into_java(&self.env);

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

    fn handle_tunnel_event(&self, event: TunnelState) {
        let java_tunnel_state = event.into_java(&self.env);

        let result = self.env.call_method_unchecked(
            self.mullvad_ipc_client,
            self.notify_tunnel_event,
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(java_tunnel_state.as_obj())],
        );

        if let Err(error) = result {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to call MullvadDaemon.notifyTunnelStateEvent")
            );
        }
    }

    fn handle_app_version_info_event(&self, app_version_info: AppVersionInfo) {
        let java_app_version_info = app_version_info.into_java(&self.env);

        let result = self.env.call_method_unchecked(
            self.mullvad_ipc_client,
            self.notify_app_version_info_event,
            JavaType::Primitive(Primitive::Void),
            &[JValue::Object(java_app_version_info.as_obj())],
        );

        if let Err(error) = result {
            log::error!(
                "{}",
                error.display_chain_with_msg(
                    "Failed to call MullvadDaemon.notifyAppVersionInfoEvent"
                )
            );
        }
    }
}
