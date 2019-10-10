#![cfg(target_os = "android")]

mod daemon_interface;
mod from_java;
mod into_java;
mod is_null;
mod jni_event_listener;
mod vpn_service_tun_provider;

use crate::{
    daemon_interface::DaemonInterface, from_java::FromJava, into_java::IntoJava,
    jni_event_listener::JniEventListener, vpn_service_tun_provider::VpnServiceTunProvider,
};
use jni::{
    objects::{GlobalRef, JObject, JString, JValue},
    sys::{jboolean, JNI_FALSE, JNI_TRUE},
    JNIEnv,
};
use lazy_static::lazy_static;
use mullvad_daemon::{logging, version, Daemon, DaemonCommandSender};
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{mpsc, Once},
    thread,
};
use talpid_types::ErrorExt;

const LOG_FILENAME: &str = "daemon.log";

const CLASSES_TO_LOAD: &[&str] = &[
    "java/lang/Boolean",
    "java/net/InetAddress",
    "java/net/InetSocketAddress",
    "java/util/ArrayList",
    "net/mullvad/mullvadvpn/model/AccountData",
    "net/mullvad/mullvadvpn/model/ActionAfterDisconnect$Block",
    "net/mullvad/mullvadvpn/model/ActionAfterDisconnect$Nothing",
    "net/mullvad/mullvadvpn/model/ActionAfterDisconnect$Reconnect",
    "net/mullvad/mullvadvpn/model/BlockReason$AuthFailed",
    "net/mullvad/mullvadvpn/model/BlockReason$Ipv6Unavailable",
    "net/mullvad/mullvadvpn/model/BlockReason$SetFirewallPolicyError",
    "net/mullvad/mullvadvpn/model/BlockReason$SetDnsError",
    "net/mullvad/mullvadvpn/model/BlockReason$StartTunnelError",
    "net/mullvad/mullvadvpn/model/BlockReason$ParameterGeneration",
    "net/mullvad/mullvadvpn/model/BlockReason$IsOffline",
    "net/mullvad/mullvadvpn/model/BlockReason$TapAdapterProblem",
    "net/mullvad/mullvadvpn/model/AppVersionInfo",
    "net/mullvad/mullvadvpn/model/Constraint$Any",
    "net/mullvad/mullvadvpn/model/Constraint$Only",
    "net/mullvad/mullvadvpn/model/Endpoint",
    "net/mullvad/mullvadvpn/model/GeoIpLocation",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$Ok",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$InvalidAccount",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$RpcError",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$OtherError",
    "net/mullvad/mullvadvpn/model/InetNetwork",
    "net/mullvad/mullvadvpn/model/KeygenEvent$NewKey",
    "net/mullvad/mullvadvpn/model/KeygenEvent$Failure",
    "net/mullvad/mullvadvpn/model/KeygenFailure$TooManyKeys",
    "net/mullvad/mullvadvpn/model/KeygenFailure$GenerationFailure",
    "net/mullvad/mullvadvpn/model/LocationConstraint$City",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Country",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Hostname",
    "net/mullvad/mullvadvpn/model/ParameterGenerationError$NoMatchingRelay",
    "net/mullvad/mullvadvpn/model/ParameterGenerationError$NoMatchingBridgeRelay",
    "net/mullvad/mullvadvpn/model/ParameterGenerationError$NoWireguardKey",
    "net/mullvad/mullvadvpn/model/ParameterGenerationError$CustomTunnelHostResultionError",
    "net/mullvad/mullvadvpn/model/PublicKey",
    "net/mullvad/mullvadvpn/model/Relay",
    "net/mullvad/mullvadvpn/model/RelayList",
    "net/mullvad/mullvadvpn/model/RelayListCity",
    "net/mullvad/mullvadvpn/model/RelayListCountry",
    "net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettings$RelayConstraints",
    "net/mullvad/mullvadvpn/model/RelaySettingsUpdate$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettingsUpdate$RelayConstraintsUpdate",
    "net/mullvad/mullvadvpn/model/Settings",
    "net/mullvad/mullvadvpn/model/TransportProtocol$Tcp",
    "net/mullvad/mullvadvpn/model/TransportProtocol$Udp",
    "net/mullvad/mullvadvpn/model/TunConfig",
    "net/mullvad/mullvadvpn/model/TunnelEndpoint",
    "net/mullvad/mullvadvpn/model/TunnelState$Blocked",
    "net/mullvad/mullvadvpn/model/TunnelState$Connected",
    "net/mullvad/mullvadvpn/model/TunnelState$Connecting",
    "net/mullvad/mullvadvpn/model/TunnelState$Disconnected",
    "net/mullvad/mullvadvpn/model/TunnelState$Disconnecting",
    "net/mullvad/mullvadvpn/MullvadDaemon",
    "net/mullvad/mullvadvpn/MullvadVpnService",
];

lazy_static! {
    static ref LOG_INIT_RESULT: Result<PathBuf, String> =
        start_logging().map_err(|error| error.display_chain());
    static ref DAEMON_INTERFACE: DaemonInterface = DaemonInterface::new();
    static ref CLASSES: RwLock<HashMap<&'static str, GlobalRef>> =
        RwLock::new(HashMap::with_capacity(CLASSES_TO_LOAD.len()));
}

static LOAD_CLASSES: Once = Once::new();

#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to create VpnService tunnel provider")]
    CreateVpnServiceTunProvider(#[error(source)] vpn_service_tun_provider::Error),

    #[error(display = "Failed to get cache directory path")]
    GetCacheDir(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to get log directory path")]
    GetLogDir(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to initialize the mullvad daemon")]
    InitializeDaemon(#[error(source)] mullvad_daemon::Error),

    #[error(display = "Failed to spawn the JNI event listener")]
    SpawnJniEventListener(#[error(source)] jni_event_listener::Error),

    #[error(display = "Failed to start logger")]
    StartLogging(#[error(source)] logging::Error),
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_initialize(
    env: JNIEnv,
    this: JObject,
    vpnService: JObject,
) {
    match *LOG_INIT_RESULT {
        Ok(ref log_dir) => {
            LOAD_CLASSES.call_once(|| load_classes(&env));

            if let Err(error) = initialize(&env, &this, &vpnService, log_dir.clone()) {
                log::error!("{}", error.display_chain());
            }
        }
        Err(ref message) => env
            .throw(message.as_str())
            .expect("Failed to throw exception"),
    }
}

fn start_logging() -> Result<PathBuf, Error> {
    let log_dir = mullvad_paths::log_dir().map_err(Error::GetLogDir)?;
    let log_file = log_dir.join(LOG_FILENAME);

    logging::init_logger(log::LevelFilter::Debug, Some(&log_file), true)
        .map_err(Error::StartLogging)?;
    log_panics::init();
    version::log_version();

    Ok(log_dir)
}

fn load_classes(env: &JNIEnv) {
    let mut classes = CLASSES.write();

    for class in CLASSES_TO_LOAD {
        classes.insert(class, load_class_reference(env, class));
    }
}

fn load_class_reference(env: &JNIEnv, name: &str) -> GlobalRef {
    let class = match env.find_class(name) {
        Ok(class) => class,
        Err(_) => panic!("Failed to find {} Java class", name),
    };

    env.new_global_ref(JObject::from(class))
        .expect("Failed to convert local reference to Java class into a global reference")
}

fn initialize(
    env: &JNIEnv,
    this: &JObject,
    vpn_service: &JObject,
    log_dir: PathBuf,
) -> Result<(), Error> {
    let tun_provider =
        VpnServiceTunProvider::new(env, vpn_service).map_err(Error::CreateVpnServiceTunProvider)?;
    let daemon_command_sender = spawn_daemon(env, this, tun_provider, log_dir)?;

    DAEMON_INTERFACE.set_command_sender(daemon_command_sender);

    Ok(())
}

fn spawn_daemon(
    env: &JNIEnv,
    this: &JObject,
    tun_provider: VpnServiceTunProvider,
    log_dir: PathBuf,
) -> Result<DaemonCommandSender, Error> {
    let listener = JniEventListener::spawn(env, this).map_err(Error::SpawnJniEventListener)?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(
        move || match create_daemon(listener, tun_provider, log_dir) {
            Ok(daemon) => {
                let _ = tx.send(Ok(daemon.command_sender()));
                match daemon.run() {
                    Ok(()) => log::info!("Mullvad daemon has stopped"),
                    Err(error) => log::error!("{}", error.display_chain()),
                }
            }
            Err(error) => {
                let _ = tx.send(Err(error));
            }
        },
    );

    rx.recv().unwrap()
}

fn create_daemon(
    listener: JniEventListener,
    tun_provider: VpnServiceTunProvider,
    log_dir: PathBuf,
) -> Result<Daemon<JniEventListener>, Error> {
    let resource_dir = mullvad_paths::get_resource_dir();
    let cache_dir = mullvad_paths::cache_dir().map_err(Error::GetCacheDir)?;

    let daemon = Daemon::start_with_event_listener_and_tun_provider(
        listener,
        tun_provider,
        Some(log_dir),
        resource_dir,
        cache_dir,
    )
    .map_err(Error::InitializeDaemon)?;

    Ok(daemon)
}

fn get_class(name: &str) -> GlobalRef {
    match CLASSES.read().get(name) {
        Some(class) => class.clone(),
        None => panic!("Class not loaded: {}", name),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_connect(_: JNIEnv, _: JObject) {
    if let Err(error) = DAEMON_INTERFACE.connect() {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to request daemon to connect")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_disconnect(_: JNIEnv, _: JObject) {
    if let Err(error) = DAEMON_INTERFACE.disconnect() {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to request daemon to disconnect")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_generateWireguardKey<'env>(
    env: JNIEnv<'env>,
    _: JObject,
) -> JObject<'env> {
    match DAEMON_INTERFACE.generate_wireguard_key() {
        Ok(keygen_event) => keygen_event.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to request to generate wireguard key")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_verifyWireguardKey<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.verify_wireguard_key() {
        Ok(key_is_valid) => env
            .new_object(
                &get_class("java/lang/Boolean"),
                "(Z)V",
                &[JValue::Bool(key_is_valid as jboolean)],
            )
            .expect("Failed to create Boolean Java object"),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to verify wireguard key")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getAccountData<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
    accountToken: JString,
) -> JObject<'env> {
    let account = String::from_java(&env, accountToken);
    let result = DAEMON_INTERFACE.get_account_data(account);

    if let Err(ref error) = &result {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to get account data")
        );
    }

    result.into_java(&env)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getWwwAuthToken<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JString<'env> {
    match DAEMON_INTERFACE.get_www_auth_token() {
        Ok(token) => {
            token.into_java(&env)
        },
        Err(err) => {
            log::error!(
                "{}",
                err.display_chain_with_msg("Failed to get WWW auth token")
            );
            String::new().into_java(&env)
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getCurrentLocation<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.get_current_location() {
        Ok(location) => location.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get current location")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getCurrentVersion<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JString<'env> {
    match DAEMON_INTERFACE.get_current_version() {
        Ok(location) => location.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get current version")
            );
            String::new().into_java(&env)
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getRelayLocations<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.get_relay_locations() {
        Ok(relay_list) => relay_list.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get relay locations")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getSettings<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.get_settings() {
        Ok(settings) => settings.into_java(&env),
        Err(error) => {
            log::error!("{}", error.display_chain_with_msg("Failed to get settings"));
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getState<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.get_state() {
        Ok(state) => state.into_java(&env),
        Err(error) => {
            log::error!("{}", error.display_chain_with_msg("Failed to get state"));
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getVersionInfo<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.get_version_info() {
        Ok(version_info) => version_info.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get version information")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_getWireguardKey<'env, 'this>(
    env: JNIEnv<'env>,
    _: JObject<'this>,
) -> JObject<'env> {
    match DAEMON_INTERFACE.get_wireguard_key() {
        Ok(key) => key.into_java(&env),
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to get wireguard key")
            );
            JObject::null()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_setAccount(
    env: JNIEnv,
    _: JObject,
    accountToken: JString,
) {
    let account = <Option<String> as FromJava>::from_java(&env, accountToken);

    if let Err(error) = DAEMON_INTERFACE.set_account(account) {
        log::error!("{}", error.display_chain_with_msg("Failed to set account"));
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_shutdown(_: JNIEnv, _: JObject) {
    if let Err(error) = DAEMON_INTERFACE.shutdown() {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to shutdown daemon thread")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_updateRelaySettings(
    env: JNIEnv,
    _: JObject,
    relaySettingsUpdate: JObject,
) {
    let update = FromJava::from_java(&env, relaySettingsUpdate);

    if let Err(error) = DAEMON_INTERFACE.update_relay_settings(update) {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to update relay settings")
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_collectReport(
    env: JNIEnv,
    _: JObject,
    outputPath: JString,
) -> jboolean {
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);

    match mullvad_problem_report::collect_report(&[], output_path, Vec::new()) {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to collect problem report")
            );
            JNI_FALSE
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_sendProblemReport(
    env: JNIEnv,
    _: JObject,
    userEmail: JString,
    userMessage: JString,
    outputPath: JString,
) -> jboolean {
    let user_email = String::from_java(&env, userEmail);
    let user_message = String::from_java(&env, userMessage);
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);

    match mullvad_problem_report::send_problem_report(&user_email, &user_message, output_path) {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to collect problem report")
            );
            JNI_FALSE
        }
    }
}
