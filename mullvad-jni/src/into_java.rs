use crate::get_class;
use ipnetwork::IpNetwork;
use jni::{
    objects::{JObject, JString, JValue},
    signature::{JavaType, Primitive},
    sys::{jboolean, jint, jshort, jsize},
    JNIEnv,
};
use mullvad_types::{
    account::AccountData,
    location::GeoIpLocation,
    relay_constraints::{Constraint, LocationConstraint, RelayConstraints, RelaySettings},
    relay_list::{Relay, RelayList, RelayListCity, RelayListCountry},
    settings::Settings,
    states::TunnelState,
    version::AppVersionInfo,
    wireguard::KeygenEvent,
    CustomTunnelEndpoint,
};
use std::{
    fmt::Debug,
    net::{IpAddr, SocketAddr},
};
use talpid_core::tunnel::tun_provider::TunConfig;
use talpid_types::{
    net::{wireguard::PublicKey, Endpoint, TransportProtocol, TunnelEndpoint},
    tunnel::{ActionAfterDisconnect, BlockReason},
};

pub trait IntoJava<'env> {
    type JavaType;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType;
}

impl<'env, T> IntoJava<'env> for Option<T>
where
    T: IntoJava<'env>,
    T::JavaType: From<JObject<'env>>,
{
    type JavaType = T::JavaType;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            Some(data) => data.into_java(env),
            None => T::JavaType::from(JObject::null()),
        }
    }
}

impl<'env> IntoJava<'env> for String {
    type JavaType = JString<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        env.new_string(&self).expect("Failed to create Java String")
    }
}

impl<'env, T> IntoJava<'env> for Vec<T>
where
    T: IntoJava<'env>,
    JObject<'env>: From<T::JavaType>,
{
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("java/util/ArrayList");
        let initial_capacity = self.len();
        let parameters = [JValue::Int(initial_capacity as jint)];

        let list_object = env
            .new_object(&class, "(I)V", &parameters)
            .expect("Failed to create ArrayList object");

        let list_class = get_class("java/util/List");
        let add_method = env
            .get_method_id(&list_class, "add", "(Ljava/lang/Object;)Z")
            .expect("Failed to get List.add(Object) method id");

        for element in self {
            let java_element = env.auto_local(JObject::from(element.into_java(env)));

            env.call_method_unchecked(
                list_object,
                add_method,
                JavaType::Primitive(Primitive::Boolean),
                &[JValue::Object(java_element.as_obj())],
            )
            .expect("Failed to add element to ArrayList");
        }

        list_object
    }
}

impl<'array, 'env> IntoJava<'env> for &'array [u8] {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let size = self.len();
        let array = env
            .new_byte_array(size as jsize)
            .expect("Failed to create a Java array of bytes");

        let data = unsafe { std::slice::from_raw_parts(self.as_ptr() as *const i8, size) };

        env.set_byte_array_region(array, 0, data)
            .expect("Failed to copy bytes to Java array");

        JObject::from(array)
    }
}

impl<'env> IntoJava<'env> for IpAddr {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("java/net/InetAddress");

        let constructor = env
            .get_static_method_id(&class, "getByAddress", "([B)Ljava/net/InetAddress;")
            .expect("Failed to get InetAddress.getByAddress method ID");

        let octet_count = if self.is_ipv4() { 4 } else { 16 };
        let octets_array = env
            .new_byte_array(octet_count)
            .expect("Failed to create byte array to store IP address");

        let octet_data: Vec<i8> = match self {
            IpAddr::V4(address) => address
                .octets()
                .into_iter()
                .map(|octet| *octet as i8)
                .collect(),
            IpAddr::V6(address) => address
                .octets()
                .into_iter()
                .map(|octet| *octet as i8)
                .collect(),
        };

        env.set_byte_array_region(octets_array, 0, &octet_data)
            .expect("Failed to copy IP address octets to byte array");

        let octets = env.auto_local(JObject::from(octets_array));
        let result = env
            .call_static_method_unchecked(
                "java/net/InetAddress",
                constructor,
                JavaType::Object("java/net/InetAddress".to_owned()),
                &[JValue::Object(octets.as_obj())],
            )
            .expect("Failed to create InetAddress Java object");

        match result {
            JValue::Object(object) => object,
            value => {
                panic!(
                    "InetAddress.getByAddress returned an invalid value: {:?}",
                    value
                );
            }
        }
    }
}

impl<'env> IntoJava<'env> for SocketAddr {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("java/net/InetSocketAddress");
        let ip_address = env.auto_local(self.ip().into_java(env));
        let port = self.port() as jint;
        let parameters = [JValue::Object(ip_address.as_obj()), JValue::Int(port)];

        env.new_object(&class, "(Ljava/net/InetAddress;I)V", &parameters)
            .expect("Failed to create InetSocketAddress Java object")
    }
}

impl<'env> IntoJava<'env> for IpNetwork {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/InetNetwork");
        let address = env.auto_local(self.ip().into_java(env));
        let prefix_length = self.prefix() as jshort;
        let parameters = [
            JValue::Object(address.as_obj()),
            JValue::Short(prefix_length),
        ];

        env.new_object(&class, "(Ljava/net/InetAddress;S)V", &parameters)
            .expect("Failed to create InetNetwork Java object")
    }
}

impl<'env> IntoJava<'env> for PublicKey {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/PublicKey");
        let key = env.auto_local(self.as_bytes().into_java(env));
        let parameters = [JValue::Object(key.as_obj())];

        env.new_object(&class, "([B)V", &parameters)
            .expect("Failed to create PublicKey Java object")
    }
}

impl<'env> IntoJava<'env> for AppVersionInfo {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/AppVersionInfo");
        let current_is_supported = self.current_is_supported as jboolean;
        let latest_stable = env.auto_local(*self.latest_stable.into_java(env));
        let latest = env.auto_local(*self.latest.into_java(env));
        let parameters = [
            JValue::Bool(current_is_supported),
            JValue::Object(latest_stable.as_obj()),
            JValue::Object(latest.as_obj()),
        ];

        env.new_object(
            &class,
            "(ZLjava/lang/String;Ljava/lang/String;)V",
            &parameters,
        )
        .expect("Failed to create AppVersionInfo Java object")
    }
}

impl<'env> IntoJava<'env> for AccountData {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/AccountData");
        let account_expiry = env.auto_local(JObject::from(self.expiry.to_string().into_java(env)));
        let parameters = [JValue::Object(account_expiry.as_obj())];

        env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
            .expect("Failed to create AccountData Java object")
    }
}

impl<'env> IntoJava<'env> for TunConfig {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/TunConfig");
        let addresses = env.auto_local(self.addresses.into_java(env));
        let dns_servers = env.auto_local(self.dns_servers.into_java(env));
        let routes = env.auto_local(self.routes.into_java(env));
        let mtu = self.mtu as jint;
        let parameters = [
            JValue::Object(addresses.as_obj()),
            JValue::Object(dns_servers.as_obj()),
            JValue::Object(routes.as_obj()),
            JValue::Int(mtu),
        ];

        env.new_object(
            &class,
            "(Ljava/util/List;Ljava/util/List;Ljava/util/List;I)V",
            &parameters,
        )
        .expect("Failed to create TunConfig Java object")
    }
}

impl<'env> IntoJava<'env> for TransportProtocol {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class_name = match self {
            TransportProtocol::Tcp => "net/mullvad/mullvadvpn/model/TransportProtocol$Tcp",
            TransportProtocol::Udp => "net/mullvad/mullvadvpn/model/TransportProtocol$Udp",
        };
        let class = get_class(class_name);

        env.new_object(&class, "()V", &[])
            .expect("Failed to create TransportProtocol sub-class variant Java object")
    }
}

impl<'env> IntoJava<'env> for Endpoint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Endpoint");
        let address = env.auto_local(self.address.into_java(env));
        let protocol = env.auto_local(self.protocol.into_java(env));
        let parameters = [
            JValue::Object(address.as_obj()),
            JValue::Object(protocol.as_obj()),
        ];

        env.new_object(
            &class,
            "(Ljava/net/InetSocketAddress;Lnet/mullvad/mullvadvpn/model/TransportProtocol;)V",
            &parameters,
        )
        .expect("Failed to create Endpoint sub-class variant Java object")
    }
}

impl<'env> IntoJava<'env> for TunnelEndpoint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/TunnelEndpoint");
        let endpoint = env.auto_local(self.endpoint.into_java(env));
        let parameters = [JValue::Object(endpoint.as_obj())];

        env.new_object(
            &class,
            "(Lnet/mullvad/mullvadvpn/model/Endpoint;)V",
            &parameters,
        )
        .expect("Failed to create TunnelEndpoint sub-class variant Java object")
    }
}

impl<'env> IntoJava<'env> for GeoIpLocation {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/GeoIpLocation");
        let country = env.auto_local(JObject::from(self.country.into_java(env)));
        let city = env.auto_local(JObject::from(self.city.into_java(env)));
        let hostname = env.auto_local(JObject::from(self.hostname.into_java(env)));
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
        .expect("Failed to create GeoIpLocation Java object")
    }
}

impl<'env> IntoJava<'env> for RelayList {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayList");
        let relay_countries = env.auto_local(self.countries.into_java(env));
        let parameters = [JValue::Object(relay_countries.as_obj())];

        env.new_object(&class, "(Ljava/util/List;)V", &parameters)
            .expect("Failed to create RelayList Java object")
    }
}

impl<'env> IntoJava<'env> for RelayListCountry {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayListCountry");
        let name = env.auto_local(JObject::from(self.name.into_java(env)));
        let code = env.auto_local(JObject::from(self.code.into_java(env)));
        let relay_cities = env.auto_local(self.cities.into_java(env));
        let parameters = [
            JValue::Object(name.as_obj()),
            JValue::Object(code.as_obj()),
            JValue::Object(relay_cities.as_obj()),
        ];

        env.new_object(
            &class,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
            &parameters,
        )
        .expect("Failed to create RelayListCountry Java object")
    }
}

impl<'env> IntoJava<'env> for RelayListCity {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayListCity");
        let name = env.auto_local(JObject::from(self.name.into_java(env)));
        let code = env.auto_local(JObject::from(self.code.into_java(env)));
        let relays = env.auto_local(self.relays.into_java(env));
        let parameters = [
            JValue::Object(name.as_obj()),
            JValue::Object(code.as_obj()),
            JValue::Object(relays.as_obj()),
        ];

        env.new_object(
            &class,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
            &parameters,
        )
        .expect("Failed to create RelayListCity Java object")
    }
}

impl<'env> IntoJava<'env> for Relay {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Relay");
        let hostname = env.auto_local(JObject::from(self.hostname.into_java(env)));
        let has_wireguard_tunnels = (!self.tunnels.wireguard.is_empty()) as jboolean;
        let parameters = [
            JValue::Object(hostname.as_obj()),
            JValue::Bool(has_wireguard_tunnels),
        ];

        env.new_object(&class, "(Ljava/lang/String;Z)V", &parameters)
            .expect("Failed to create Relay Java object")
    }
}

impl<'env, T> IntoJava<'env> for Constraint<T>
where
    T: Clone + Eq + Debug + IntoJava<'env>,
    JObject<'env>: From<T::JavaType>,
{
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            Constraint::Any => {
                let class = get_class("net/mullvad/mullvadvpn/model/Constraint$Any");

                env.new_object(&class, "()V", &[])
                    .expect("Failed to create Constraint.Any Java object")
            }
            Constraint::Only(constraint) => {
                let class = get_class("net/mullvad/mullvadvpn/model/Constraint$Only");
                let value = env.auto_local(JObject::from(constraint.into_java(env)));
                let parameters = [JValue::Object(value.as_obj())];

                env.new_object(&class, "(Ljava/lang/Object;)V", &parameters)
                    .expect("Failed to create Constraint.Only Java object")
            }
        }
    }
}

impl<'env> IntoJava<'env> for LocationConstraint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            LocationConstraint::Country(country_code) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Country");
                let country = env.auto_local(JObject::from(country_code.into_java(env)));
                let parameters = [JValue::Object(country.as_obj())];

                env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
                    .expect("Failed to create LocationConstraint.Country Java object")
            }
            LocationConstraint::City(country_code, city_code) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$City");
                let country = env.auto_local(JObject::from(country_code.into_java(env)));
                let city = env.auto_local(JObject::from(city_code.into_java(env)));
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
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Hostname");
                let country = env.auto_local(JObject::from(country_code.into_java(env)));
                let city = env.auto_local(JObject::from(city_code.into_java(env)));
                let hostname = env.auto_local(JObject::from(hostname.into_java(env)));
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
        }
    }
}

impl<'env> IntoJava<'env> for RelaySettings {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            RelaySettings::CustomTunnelEndpoint(endpoint) => endpoint.into_java(env),
            RelaySettings::Normal(relay_constraints) => relay_constraints.into_java(env),
        }
    }
}

impl<'env> IntoJava<'env> for CustomTunnelEndpoint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint");

        env.new_object(&class, "()V", &[])
            .expect("Failed to create CustomTunnelEndpoint Java object")
    }
}

impl<'env> IntoJava<'env> for KeygenEvent {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            KeygenEvent::NewKey(public_key) => {
                let class = get_class("net/mullvad/mullvadvpn/model/KeygenEvent$NewKey");
                let java_public_key = env.auto_local(public_key.into_java(env));
                let parameters = [JValue::Object(java_public_key.as_obj())];

                env.new_object(
                    &class,
                    "(Lnet/mullvad/mullvadvpn/model/PublicKey;)V",
                    &parameters,
                )
                .expect("Failed to create KeygenEvent.NewKey Java object")
            }
            KeygenEvent::TooManyKeys => {
                let class = get_class("net/mullvad/mullvadvpn/model/KeygenEvent$TooManyKeys");

                env.new_object(&class, "()V", &[])
                    .expect("Failed to create KeygenEvent.TooManyKeys Java object")
            }
            KeygenEvent::GenerationFailure => {
                let class = get_class("net/mullvad/mullvadvpn/model/KeygenEvent$GenerationFailure");

                env.new_object(&class, "()V", &[])
                    .expect("Failed to create KeygenEvent.GenerationFailure Java object")
            }
        }
    }
}

impl<'env> IntoJava<'env> for RelayConstraints {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelaySettings$RelayConstraints");
        let location = env.auto_local(self.location.into_java(env));
        let parameters = [JValue::Object(location.as_obj())];

        env.new_object(
            &class,
            "(Lnet/mullvad/mullvadvpn/model/Constraint;)V",
            &parameters,
        )
        .expect("Failed to create RelaySettings.RelayConstraints Java object")
    }
}

impl<'env> IntoJava<'env> for Settings {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Settings");
        let account_token = env.auto_local(JObject::from(self.get_account_token().into_java(env)));
        let relay_settings = env.auto_local(self.get_relay_settings().into_java(env));
        let parameters = [
            JValue::Object(account_token.as_obj()),
            JValue::Object(relay_settings.as_obj()),
        ];

        env.new_object(
            &class,
            "(Ljava/lang/String;Lnet/mullvad/mullvadvpn/model/RelaySettings;)V",
            &parameters,
        )
        .expect("Failed to create Settings Java object")
    }
}

impl<'env> IntoJava<'env> for ActionAfterDisconnect {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let variant = match self {
            ActionAfterDisconnect::Nothing => "Nothing",
            ActionAfterDisconnect::Block => "Block",
            ActionAfterDisconnect::Reconnect => "Reconnect",
        };
        let class_name = format!(
            "net/mullvad/mullvadvpn/model/ActionAfterDisconnect${}",
            variant
        );
        let class = get_class(&class_name);

        env.new_object(&class, "()V", &[])
            .expect("Failed to create ActionAfterDisconnect sub-class variant Java object")
    }
}

impl<'env> IntoJava<'env> for BlockReason {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let variant = match self {
            BlockReason::AuthFailed(reason) => {
                let class = get_class("net/mullvad/mullvadvpn/model/BlockReason$AuthFailed");
                let reason = env.auto_local(JObject::from(reason.into_java(env)));
                let parameters = [JValue::Object(reason.as_obj())];

                return env
                    .new_object(&class, "(Ljava/lang/String;)V", &parameters)
                    .expect("Failed to create BlockReason.AuthFailed Java object");
            }
            BlockReason::Ipv6Unavailable => "Ipv6Unavailable",
            BlockReason::SetFirewallPolicyError => "SetFirewallPolicyError",
            BlockReason::SetDnsError => "SetDnsError",
            BlockReason::StartTunnelError => "StartTunnelError",
            BlockReason::NoMatchingRelay => "NoMatchingRelay",
            BlockReason::IsOffline => "IsOffline",
            BlockReason::TapAdapterProblem => "TapAdapterProblem",
        };
        let class_name = format!("net/mullvad/mullvadvpn/model/BlockReason${}", variant);
        let class = get_class(&class_name);

        env.new_object(&class, "()V", &[])
            .expect("Failed to create BlockReason sub-class variant Java object")
    }
}

impl<'env> IntoJava<'env> for TunnelState {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            TunnelState::Disconnected => {
                let class = get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnected");

                env.new_object(&class, "()V", &[])
            }
            TunnelState::Connecting { location, .. } => {
                let class = get_class("net/mullvad/mullvadvpn/model/TunnelState$Connecting");
                let location = env.auto_local(location.into_java(env));
                let parameters = [JValue::Object(location.as_obj())];
                let signature = "(Lnet/mullvad/mullvadvpn/model/GeoIpLocation;)V";

                env.new_object(&class, signature, &parameters)
            }
            TunnelState::Connected { location, .. } => {
                let class = get_class("net/mullvad/mullvadvpn/model/TunnelState$Connected");
                let location = env.auto_local(location.into_java(env));
                let parameters = [JValue::Object(location.as_obj())];
                let signature = "(Lnet/mullvad/mullvadvpn/model/GeoIpLocation;)V";

                env.new_object(&class, signature, &parameters)
            }
            TunnelState::Disconnecting(action_after_disconnect) => {
                let class = get_class("net/mullvad/mullvadvpn/model/TunnelState$Disconnecting");
                let after_disconnect = env.auto_local(action_after_disconnect.into_java(env));
                let parameters = [JValue::Object(after_disconnect.as_obj())];
                let signature = "(Lnet/mullvad/mullvadvpn/model/ActionAfterDisconnect;)V";

                env.new_object(&class, signature, &parameters)
            }
            TunnelState::Blocked(block_reason) => {
                let class = get_class("net/mullvad/mullvadvpn/model/TunnelState$Blocked");
                let reason = env.auto_local(block_reason.into_java(env));
                let parameters = [JValue::Object(reason.as_obj())];
                let signature = "(Lnet/mullvad/mullvadvpn/model/BlockReason;)V";

                env.new_object(&class, signature, &parameters)
            }
        }
        .expect("Failed to create TunnelState sub-class variant Java object")
    }
}
