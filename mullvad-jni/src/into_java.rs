use crate::daemon_interface;
use jnix::{
    jni::objects::{AutoLocal, JValue},
    JnixEnv,
};
use mullvad_types::{
    account::AccountData, relay_constraints::Constraint, relay_list::RelayList, settings::Settings,
    states::TunnelState, version::AppVersionInfo, wireguard::KeygenEvent,
};
use std::fmt::Debug;
use talpid_core::tunnel::tun_provider::TunConfig;
use talpid_types::{
    net::TunnelEndpoint,
    tunnel::{ActionAfterDisconnect, BlockReason, ParameterGenerationError},
};

pub trait IntoJava<'borrow, 'env: 'borrow> {
    type JavaType;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType;
}

macro_rules! wrap_jnix_into_java {
    ( $type:ty $( where $param:ident : $( $constraints:tt )* )* ) => {
        impl<'borrow, 'env, $( $param ),* > IntoJava<'borrow, 'env> for $type
        where
            'env: 'borrow,
            $( $param: $( $constraints )* ),*
        {
            type JavaType = <$type as jnix::IntoJava<'borrow, 'env>>::JavaType;

            fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
                jnix::IntoJava::into_java(self, env)
            }
        }
    };
}

wrap_jnix_into_java!(
    Option<T> where T: jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

wrap_jnix_into_java!(String);

wrap_jnix_into_java!(
    Vec<T> where T: jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

impl<'array, 'borrow, 'env> IntoJava<'borrow, 'env> for &'array [u8]
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        jnix::IntoJava::into_java(self, env)
    }
}

wrap_jnix_into_java!(AppVersionInfo);
wrap_jnix_into_java!(AccountData);
wrap_jnix_into_java!(TunConfig);
wrap_jnix_into_java!(TunnelEndpoint);
wrap_jnix_into_java!(RelayList);

wrap_jnix_into_java!(Constraint<T>
    where
        T: Clone + Eq + Debug + jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

wrap_jnix_into_java!(KeygenEvent);
wrap_jnix_into_java!(Settings);

impl<'borrow, 'env> IntoJava<'borrow, 'env> for ActionAfterDisconnect
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let variant = match self {
            ActionAfterDisconnect::Nothing => "Nothing",
            ActionAfterDisconnect::Block => "Block",
            ActionAfterDisconnect::Reconnect => "Reconnect",
        };
        let class_name = format!(
            "net/mullvad/talpid/tunnel/ActionAfterDisconnect${}",
            variant
        );
        let class = env.get_class(&class_name);

        env.auto_local(
            env.new_object(&class, "()V", &[])
                .expect("Failed to create ActionAfterDisconnect sub-class variant Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for BlockReason
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let variant = match self {
            BlockReason::AuthFailed(reason) => {
                let class = env.get_class("net/mullvad/talpid/tunnel/BlockReason$AuthFailed");
                let reason = reason.into_java(env);
                let parameters = [JValue::Object(reason.as_obj())];

                return env.auto_local(
                    env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
                        .expect("Failed to create BlockReason.AuthFailed Java object"),
                );
            }
            BlockReason::Ipv6Unavailable => "Ipv6Unavailable",
            BlockReason::SetFirewallPolicyError => "SetFirewallPolicyError",
            BlockReason::SetDnsError => "SetDnsError",
            BlockReason::StartTunnelError => "StartTunnelError",
            BlockReason::TunnelParameterError(reason) => {
                let class =
                    env.get_class("net/mullvad/talpid/tunnel/BlockReason$ParameterGeneration");
                let reason = reason.into_java(env);
                let parameters = [JValue::Object(reason.as_obj())];
                return env.auto_local(
                    env.new_object(
                        &class,
                        "(Lnet/mullvad/talpid/tunnel/ParameterGenerationError;)V",
                        &parameters,
                    )
                    .expect("Failed to create BlockReason.ParameterGeneration Java object"),
                );
            }
            BlockReason::IsOffline => "IsOffline",
            BlockReason::TapAdapterProblem => "TapAdapterProblem",
        };
        let class_name = format!("net/mullvad/talpid/tunnel/BlockReason${}", variant);
        let class = env.get_class(&class_name);

        env.auto_local(
            env.new_object(&class, "()V", &[])
                .expect("Failed to create BlockReason sub-class variant Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for ParameterGenerationError
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class_variant = match self {
            ParameterGenerationError::NoMatchingRelay => "NoMatchingRelay",
            ParameterGenerationError::NoMatchingBridgeRelay => "NoMatchingBridgeRelay ",
            ParameterGenerationError::NoWireguardKey => "NoWireguardKey",
            ParameterGenerationError::CustomTunnelHostResultionError => {
                "CustomTunnelHostResultionError"
            }
        };
        let class_name = format!(
            "net/mullvad/talpid/tunnel/ParameterGenerationError${}",
            class_variant
        );
        let class = env.get_class(&class_name);
        env.auto_local(
            env.new_object(&class, "()V", &[])
                .expect("Failed to create ParameterGenerationError sub-class variant Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for TunnelState
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        env.auto_local(match self {
            TunnelState::Disconnected => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnected");

                env.new_object(&class, "()V", &[])
            }
            TunnelState::Connecting { endpoint, location } => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Connecting");
                let endpoint = endpoint.into_java(env);
                let location = location.into_java(env);
                let parameters = [
                    JValue::Object(endpoint.as_obj()),
                    JValue::Object(location.as_obj()),
                ];
                let signature =
                    "(Lnet/mullvad/talpid/net/TunnelEndpoint;Lnet/mullvad/mullvadvpn/model/GeoIpLocation;)V";

                env.new_object(&class, signature, &parameters)
            }
            TunnelState::Connected { endpoint, location } => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Connected");
                let endpoint = endpoint.into_java(env);
                let location = location.into_java(env);
                let parameters = [
                    JValue::Object(endpoint.as_obj()),
                    JValue::Object(location.as_obj()),
                ];
                let signature =
                    "(Lnet/mullvad/talpid/net/TunnelEndpoint;Lnet/mullvad/mullvadvpn/model/GeoIpLocation;)V";

                env.new_object(&class, signature, &parameters)
            }
            TunnelState::Disconnecting(action_after_disconnect) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnecting");
                let after_disconnect = action_after_disconnect.into_java(env);
                let parameters = [JValue::Object(after_disconnect.as_obj())];
                let signature = "(Lnet/mullvad/talpid/tunnel/ActionAfterDisconnect;)V";

                env.new_object(&class, signature, &parameters)
            }
            TunnelState::Blocked(block_reason) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/TunnelState$Blocked");
                let reason = block_reason.into_java(env);
                let parameters = [JValue::Object(reason.as_obj())];
                let signature = "(Lnet/mullvad/talpid/tunnel/BlockReason;)V";

                env.new_object(&class, signature, &parameters)
            }
        }
        .expect("Failed to create TunnelState sub-class variant Java object"))
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for Result<AccountData, daemon_interface::Error>
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        env.auto_local(match self {
            Ok(data) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/GetAccountDataResult$Ok");
                let java_account_data = data.into_java(&env);
                let parameters = [JValue::Object(java_account_data.as_obj())];

                env.new_object(
                    &class,
                    "(Lnet/mullvad/mullvadvpn/model/AccountData;)V",
                    &parameters,
                )
                .expect("Failed to create GetAccountDataResult.Ok Java object")
            }
            Err(error) => {
                let class_name = match error {
                    daemon_interface::Error::RpcError(jsonrpc_client_core::Error(
                        jsonrpc_client_core::ErrorKind::JsonRpcError(jsonrpc_core::Error {
                            code: jsonrpc_core::ErrorCode::ServerError(-200),
                            ..
                        }),
                        _,
                    )) => "net/mullvad/mullvadvpn/model/GetAccountDataResult$InvalidAccount",
                    daemon_interface::Error::RpcError(_) => {
                        "net/mullvad/mullvadvpn/model/GetAccountDataResult$RpcError"
                    }
                    _ => "net/mullvad/mullvadvpn/model/GetAccountDataResult$OtherError",
                };
                let class = env.get_class(class_name);

                env.new_object(&class, "()V", &[])
                    .expect("Failed to create a GetAccountDataResult error sub-class Java object")
            }
        })
    }
}
