use crate::location::GeoIpLocation;
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::fmt;
use talpid_types::{
    net::TunnelEndpoint,
    tunnel::{ActionAfterDisconnect, ErrorState},
};

/// Represents the state the client strives towards.
/// When in `Secured`, the client should keep the computer from leaking and try to
/// establish a VPN tunnel if it is not up.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetState {
    Unsecured,
    Secured,
}

impl fmt::Display for TargetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetState::Unsecured => "Unsecured".fmt(f),
            TargetState::Secured => "Secured".fmt(f),
        }
    }
}

/// Represents the state the client tunnel is in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state", content = "details")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum TunnelState {
    Disconnected {
        location: Option<GeoIpLocation>,
        /// Whether internet access is blocked due to lockdown mode
        locked_down: bool,
    },
    Connecting {
        endpoint: TunnelEndpoint,
        location: Option<GeoIpLocation>,
    },
    Connected {
        endpoint: TunnelEndpoint,
        location: Option<GeoIpLocation>,
    },
    Disconnecting(ActionAfterDisconnect),
    Error(ErrorState),
}

impl TunnelState {
    /// Returns true if the tunnel state is in the error state.
    pub fn is_in_error_state(&self) -> bool {
        matches!(self, TunnelState::Error(_))
    }

    /// Returns true if the tunnel state is in the connected state.
    pub fn is_connected(&self) -> bool {
        matches!(self, TunnelState::Connected { .. })
    }

    /// Returns true if the tunnel state is in the disconnected state.
    pub fn is_disconnected(&self) -> bool {
        matches!(self, TunnelState::Disconnected { .. })
    }
}

// #[cfg(target_os = "android")]
impl<'borrow, 'env> jnix::IntoJava<'borrow, 'env> for TunnelState
where
    'env: 'borrow,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/TunnelState;";
    type JavaType = jnix::jni::objects::AutoLocal<'env, 'borrow>;
    #[allow(non_snake_case)]
    fn into_java(self, env: &'borrow jnix::JnixEnv<'env>) -> Self::JavaType {
        match self {
            Self::Disconnected(_0) => {
                let _source__0 = _0;
                let _converted__0 = _source__0;
                let _signature__0 = _converted__0.jni_signature();
                let _final__0 = _converted__0.into_java(env);
                let mut constructor_signature =
                    String::with_capacity(1 + _signature__0.as_bytes().len() + 2);
                constructor_signature.push_str("(");
                constructor_signature.push_str(_signature__0);
                constructor_signature.push_str(")V");
                let parameters = [jnix::AsJValue::as_jvalue(&_final__0)];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnected");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Disconnected Rust type into net.mullvad.mullvadvpn.model.TunnelState.Disconnected Java object",
                    );
                env.auto_local(object)
            }
            Self::Connecting { endpoint, location } => {
                let _source_endpoint = endpoint;
                let _source_location = location;
                let _converted_endpoint = _source_endpoint;
                let _signature_endpoint = _converted_endpoint.jni_signature();
                let _final_endpoint = _converted_endpoint.into_java(env);
                let _converted_location = _source_location;
                let _signature_location = _converted_location.jni_signature();
                let _final_location = _converted_location.into_java(env);
                let mut constructor_signature = String::with_capacity(
                    1 + _signature_endpoint.as_bytes().len()
                        + _signature_location.as_bytes().len()
                        + 2,
                );
                constructor_signature.push_str("(");
                constructor_signature.push_str(_signature_endpoint);
                constructor_signature.push_str(_signature_location);
                constructor_signature.push_str(")V");
                let parameters = [
                    jnix::AsJValue::as_jvalue(&_final_endpoint),
                    jnix::AsJValue::as_jvalue(&_final_location),
                ];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Connecting");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Connecting Rust type into net.mullvad.mullvadvpn.model.TunnelState.Connecting Java object",
                    );
                env.auto_local(object)
            }
            Self::Connected { endpoint, location } => {
                let _source_endpoint = endpoint;
                let _source_location = location;
                let _converted_endpoint = _source_endpoint;
                let _signature_endpoint = _converted_endpoint.jni_signature();
                let _final_endpoint = _converted_endpoint.into_java(env);
                let _converted_location = _source_location;
                let _signature_location = _converted_location.jni_signature();
                let _final_location = _converted_location.into_java(env);
                let mut constructor_signature = String::with_capacity(
                    1 + _signature_endpoint.as_bytes().len()
                        + _signature_location.as_bytes().len()
                        + 2,
                );
                constructor_signature.push_str("(");
                constructor_signature.push_str(_signature_endpoint);
                constructor_signature.push_str(_signature_location);
                constructor_signature.push_str(")V");
                let parameters = [
                    jnix::AsJValue::as_jvalue(&_final_endpoint),
                    jnix::AsJValue::as_jvalue(&_final_location),
                ];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Connected");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Connected Rust type into net.mullvad.mullvadvpn.model.TunnelState.Connected Java object",
                    );
                env.auto_local(object)
            }
            Self::Disconnecting(_0) => {
                let _source__0 = _0;
                let _converted__0 = _source__0;
                let _signature__0 = _converted__0.jni_signature();
                let _final__0 = _converted__0.into_java(env);
                let mut constructor_signature =
                    String::with_capacity(1 + _signature__0.as_bytes().len() + 2);
                constructor_signature.push_str("(");
                constructor_signature.push_str(_signature__0);
                constructor_signature.push_str(")V");
                let parameters = [jnix::AsJValue::as_jvalue(&_final__0)];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnecting");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Disconnecting Rust type into net.mullvad.mullvadvpn.model.TunnelState.Disconnecting Java object",
                    );
                env.auto_local(object)
            }
            Self::Error(_0) => {
                let _source__0 = _0;
                let _converted__0 = _source__0;
                let _signature__0 = _converted__0.jni_signature();
                let _final__0 = _converted__0.into_java(env);
                let mut constructor_signature =
                    String::with_capacity(1 + _signature__0.as_bytes().len() + 2);
                constructor_signature.push_str("(");
                constructor_signature.push_str(_signature__0);
                constructor_signature.push_str(")V");
                let parameters = [jnix::AsJValue::as_jvalue(&_final__0)];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Error");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Error Rust type into net.mullvad.mullvadvpn.model.TunnelState.Error Java object",
                    );
                env.auto_local(object)
            }
        }
    }
}
