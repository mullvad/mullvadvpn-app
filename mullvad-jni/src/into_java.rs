use crate::daemon_interface;
use jnix::{
    jni::{
        objects::{AutoLocal, JList, JObject, JValue},
        sys::{jboolean, jint},
    },
    JnixEnv,
};
use mullvad_types::{
    account::AccountData,
    location::GeoIpLocation,
    relay_constraints::{Constraint, LocationConstraint, RelayConstraints, RelaySettings},
    relay_list::{Relay, RelayList, RelayListCity, RelayListCountry},
    settings::Settings,
    states::TunnelState,
    version::AppVersionInfo,
    wireguard::{KeygenEvent, PublicKey},
    CustomTunnelEndpoint,
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

impl<'borrow, 'env, T> IntoJava<'borrow, 'env> for Option<T>
where
    'env: 'borrow,
    T: IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        match self {
            Some(data) => data.into_java(env),
            None => env.auto_local(JObject::null()),
        }
    }
}

wrap_jnix_into_java!(String);

impl<'borrow, 'env, T> IntoJava<'borrow, 'env> for Vec<T>
where
    'env: 'borrow,
    T: IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("java/util/ArrayList");
        let initial_capacity = self.len();
        let parameters = [JValue::Int(initial_capacity as jint)];

        let list_object = env
            .new_object(&class, "(I)V", &parameters)
            .expect("Failed to create ArrayList object");

        let list =
            JList::from_env(env, list_object).expect("Failed to create JList from ArrayList");

        for element in self {
            let java_element = element.into_java(env);

            list.add(java_element.as_obj())
                .expect("Failed to add element to ArrayList");
        }

        env.auto_local(list_object)
    }
}

impl<'array, 'borrow, 'env> IntoJava<'borrow, 'env> for &'array [u8]
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        jnix::IntoJava::into_java(self, env)
    }
}


impl<'borrow, 'env> IntoJava<'borrow, 'env> for PublicKey
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/PublicKey");
        let key = self.key.as_bytes().into_java(env);
        let date_created = self.created.to_string().into_java(env);
        let parameters = [
            JValue::Object(key.as_obj()),
            JValue::Object(date_created.as_obj()),
        ];

        env.auto_local(
            env.new_object(&class, "([BLjava/lang/String;)V", &parameters)
                .expect("Failed to create PublicKey Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for AppVersionInfo
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/AppVersionInfo");
        let current_is_supported = self.current_is_supported as jboolean;
        let current_is_outdated = self.current_is_outdated as jboolean;
        let latest_stable = self.latest_stable.into_java(env);
        let latest = self.latest.into_java(env);
        let parameters = [
            JValue::Bool(current_is_supported),
            JValue::Bool(current_is_outdated),
            JValue::Object(latest_stable.as_obj()),
            JValue::Object(latest.as_obj()),
        ];

        env.auto_local(
            env.new_object(
                &class,
                "(ZZLjava/lang/String;Ljava/lang/String;)V",
                &parameters,
            )
            .expect("Failed to create AppVersionInfo Java object"),
        )
    }
}

wrap_jnix_into_java!(AccountData);
wrap_jnix_into_java!(TunConfig);
wrap_jnix_into_java!(TunnelEndpoint);
wrap_jnix_into_java!(GeoIpLocation);

impl<'borrow, 'env> IntoJava<'borrow, 'env> for RelayList
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/RelayList");
        let relay_countries = self.countries.into_java(env);
        let parameters = [JValue::Object(relay_countries.as_obj())];

        env.auto_local(
            env.new_object(&class, "(Ljava/util/List;)V", &parameters)
                .expect("Failed to create RelayList Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for RelayListCountry
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/RelayListCountry");
        let name = self.name.into_java(env);
        let code = self.code.into_java(env);
        let relay_cities = self.cities.into_java(env);
        let parameters = [
            JValue::Object(name.as_obj()),
            JValue::Object(code.as_obj()),
            JValue::Object(relay_cities.as_obj()),
        ];

        env.auto_local(
            env.new_object(
                &class,
                "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
                &parameters,
            )
            .expect("Failed to create RelayListCountry Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for RelayListCity
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/RelayListCity");
        let name = self.name.into_java(env);
        let code = self.code.into_java(env);
        let relays = self.relays.into_java(env);
        let parameters = [
            JValue::Object(name.as_obj()),
            JValue::Object(code.as_obj()),
            JValue::Object(relays.as_obj()),
        ];

        env.auto_local(
            env.new_object(
                &class,
                "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
                &parameters,
            )
            .expect("Failed to create RelayListCity Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for Relay
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/Relay");
        let hostname = self.hostname.into_java(env);
        let has_wireguard_tunnels = (!self.tunnels.wireguard.is_empty()) as jboolean;
        let parameters = [
            JValue::Object(hostname.as_obj()),
            JValue::Bool(has_wireguard_tunnels),
            JValue::Bool(self.active as jboolean),
        ];

        env.auto_local(
            env.new_object(&class, "(Ljava/lang/String;ZZ)V", &parameters)
                .expect("Failed to create Relay Java object"),
        )
    }
}

impl<'borrow, 'env, T> IntoJava<'borrow, 'env> for Constraint<T>
where
    'env: 'borrow,
    T: Clone + Eq + Debug + IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        env.auto_local(match self {
            Constraint::Any => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/Constraint$Any");

                env.new_object(&class, "()V", &[])
                    .expect("Failed to create Constraint.Any Java object")
            }
            Constraint::Only(constraint) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/Constraint$Only");
                let value = constraint.into_java(env);
                let parameters = [JValue::Object(value.as_obj())];

                env.new_object(&class, "(Ljava/lang/Object;)V", &parameters)
                    .expect("Failed to create Constraint.Only Java object")
            }
        })
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for LocationConstraint
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        env.auto_local(match self {
            LocationConstraint::Country(country_code) => {
                let class =
                    env.get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Country");
                let country = country_code.into_java(env);
                let parameters = [JValue::Object(country.as_obj())];

                env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
                    .expect("Failed to create LocationConstraint.Country Java object")
            }
            LocationConstraint::City(country_code, city_code) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/LocationConstraint$City");
                let country = country_code.into_java(env);
                let city = city_code.into_java(env);
                let parameters = [
                    JValue::Object(country.as_obj()),
                    JValue::Object(city.as_obj()),
                ];

                env.new_object(
                    &class,
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    &parameters,
                )
                .expect("Failed to create LocationConstraint.City Java object")
            }
            LocationConstraint::Hostname(country_code, city_code, hostname) => {
                let class =
                    env.get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Hostname");
                let country = country_code.into_java(env);
                let city = city_code.into_java(env);
                let hostname = hostname.into_java(env);
                let parameters = [
                    JValue::Object(country.as_obj()),
                    JValue::Object(city.as_obj()),
                    JValue::Object(hostname.as_obj()),
                ];

                env.new_object(
                    &class,
                    "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                    &parameters,
                )
                .expect("Failed to create LocationConstraint.Hostname Java object")
            }
        })
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for RelaySettings
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        match self {
            RelaySettings::CustomTunnelEndpoint(endpoint) => endpoint.into_java(env),
            RelaySettings::Normal(relay_constraints) => relay_constraints.into_java(env),
        }
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for CustomTunnelEndpoint
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class =
            env.get_class("net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint");

        env.auto_local(
            env.new_object(&class, "()V", &[])
                .expect("Failed to create CustomTunnelEndpoint Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for KeygenEvent
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        env.auto_local(match self {
            KeygenEvent::NewKey(public_key) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/KeygenEvent$NewKey");
                let java_public_key = public_key.into_java(env);

                let parameters = [
                    JValue::Object(java_public_key.as_obj()),
                    JValue::Object(JObject::null()),
                    JValue::Object(JObject::null()),
                ];

                env.new_object(
                    &class,
                    "(Lnet/mullvad/mullvadvpn/model/PublicKey;Ljava/lang/Boolean;Lnet/mullvad/mullvadvpn/model/KeygenFailure;)V",
                    &parameters,
                )
                .expect("Failed to create KeygenEvent.NewKey Java object")
            }
            KeygenEvent::TooManyKeys => {
                let failure_class =
                    env.get_class("net/mullvad/mullvadvpn/model/KeygenFailure$TooManyKeys");

                let failure = env
                    .new_object(&failure_class, "()V", &[])
                    .expect("Failed to create KeygenFailure.TooManyKeys Java object");

                let class = env.get_class("net/mullvad/mullvadvpn/model/KeygenEvent$Failure");
                env.new_object(
                    &class,
                    "(Lnet/mullvad/mullvadvpn/model/KeygenFailure;)V",
                    &[JValue::Object(failure)],
                )
                .expect("Failed to create KeygenEvent.Failure Java object")
            }
            KeygenEvent::GenerationFailure => {
                let failure_class =
                    env.get_class("net/mullvad/mullvadvpn/model/KeygenFailure$GenerationFailure");
                let failure = env
                    .new_object(&failure_class, "()V", &[])
                    .expect("Failed to create KeygenFailure.GenerationFailure Java object");

                let class = env.get_class("net/mullvad/mullvadvpn/model/KeygenEvent$Failure");
                env.new_object(
                    &class,
                    "(Lnet/mullvad/mullvadvpn/model/KeygenFailure;)V",
                    &[JValue::Object(failure)],
                )
                .expect("Failed to create KeygenEvent.GenerationFailure Java object")
            }
        })
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for RelayConstraints
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/RelaySettings$RelayConstraints");
        let location = self.location.into_java(env);
        let parameters = [JValue::Object(location.as_obj())];

        env.auto_local(
            env.new_object(
                &class,
                "(Lnet/mullvad/mullvadvpn/model/Constraint;)V",
                &parameters,
            )
            .expect("Failed to create RelaySettings.RelayConstraints Java object"),
        )
    }
}

impl<'borrow, 'env> IntoJava<'borrow, 'env> for Settings
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        let class = env.get_class("net/mullvad/mullvadvpn/model/Settings");
        let account_token = self.get_account_token().into_java(env);
        let relay_settings = self.get_relay_settings().into_java(env);
        let parameters = [
            JValue::Object(account_token.as_obj()),
            JValue::Object(relay_settings.as_obj()),
        ];

        env.auto_local(
            env.new_object(
                &class,
                "(Ljava/lang/String;Lnet/mullvad/mullvadvpn/model/RelaySettings;)V",
                &parameters,
            )
            .expect("Failed to create Settings Java object"),
        )
    }
}

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
