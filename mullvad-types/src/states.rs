use crate::location::GeoIpLocation;
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

#[cfg(target_os = "android")]
// Here we manually implement the `IntoJava` trait of jnix to skip the `locked_down` field of
// `TunnelState::Disconnected`. The derive macro currently does not support skipping fields in
// struct variants of enums. It was decided that this solution is the preferred to updating the
// macro since the jnix crate will be dropped once android implements gRPC.
impl<'borrow, 'env> jnix::IntoJava<'borrow, 'env> for TunnelState
where
    'env: 'borrow,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/TunnelState;";
    type JavaType = jnix::jni::objects::AutoLocal<'env, 'borrow>;
    #[allow(non_snake_case)]
    fn into_java(self, env: &'borrow jnix::JnixEnv<'env>) -> Self::JavaType {
        match self {
            Self::Disconnected {
                location,
                locked_down: _,
            } => {
                let location_signature = location.jni_signature();
                let location_java = location.into_java(env);

                let constructor_signature = format!("({location_signature})V");
                let parameters = [jnix::AsJValue::as_jvalue(&location_java)];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnected");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Disconnected Rust type into net.mullvad.mullvadvpn.model.TunnelState.Disconnected Java object",
                    );
                env.auto_local(object)
            }
            Self::Connecting { endpoint, location } => {
                let endpoint_signature = endpoint.jni_signature();
                let endpoint_java = endpoint.into_java(env);
                let location_signature = location.jni_signature();
                let location_java = location.into_java(env);

                let constructor_signature = format!("({endpoint_signature}{location_signature})V");
                let parameters = [
                    jnix::AsJValue::as_jvalue(&endpoint_java),
                    jnix::AsJValue::as_jvalue(&location_java),
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
                let endpoint_signature = endpoint.jni_signature();
                let endpoint_java = endpoint.into_java(env);
                let location_signature = location.jni_signature();
                let location_java = location.into_java(env);

                let constructor_signature = format!("({endpoint_signature}{location_signature})V");
                let parameters = [
                    jnix::AsJValue::as_jvalue(&endpoint_java),
                    jnix::AsJValue::as_jvalue(&location_java),
                ];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Connected");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Connected Rust type into net.mullvad.mullvadvpn.model.TunnelState.Connected Java object",
                    );
                env.auto_local(object)
            }
            Self::Disconnecting(action_after_disconnect) => {
                let action_after_disconnect_signature = action_after_disconnect.jni_signature();
                let action_after_disconnect_java = action_after_disconnect.into_java(env);

                let constructor_signature = format!("({action_after_disconnect_signature})V");
                let parameters = [jnix::AsJValue::as_jvalue(&action_after_disconnect_java)];
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnecting");
                let object = env
                    .new_object(&class, constructor_signature, &parameters)
                    .expect(
                        "Failed to convert TunnelState::Disconnecting Rust type into net.mullvad.mullvadvpn.model.TunnelState.Disconnecting Java object",
                    );
                env.auto_local(object)
            }
            Self::Error(error_state) => {
                let error_state_signature = error_state.jni_signature();
                let error_state_java = error_state.into_java(env);

                let constructor_signature = format!("({error_state})V");
                let parameters = [jnix::AsJValue::as_jvalue(&error_state_java)];
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
