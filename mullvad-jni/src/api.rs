use jnix::{
    jni::{
        objects::JObject,
        signature::{JavaType, Primitive},
    },
    FromJava, JnixEnv,
};
use std::net::{SocketAddr, ToSocketAddrs};

pub fn api_endpoint_from_java(
    env: &JnixEnv<'_>,
    object: JObject<'_>,
) -> Option<mullvad_api::ApiEndpoint> {
    /// Check for default instead of null
    let is_default = env.call_method(object, "isDefault", "()Z", &[])
            .expect("missing ApiEndpoint.isDefault")
            .z()
            .expect("ApiEndpoint.isDefault is not a bool");
    if is_default {
        return None;
    }

    let mut endpoint = mullvad_api::ApiEndpoint::from_env_vars();

    endpoint.address = Some(
       try_socketaddr_from_java(env, object).expect("received unresolved InetSocketAddress"),
    );

    endpoint.host = try_hostname_from_java(env, object);
    #[cfg(feature = "api-override")]
    {
        endpoint.disable_address_cache = env
            .call_method(object, "component3", "()Z", &[])
            .expect("missing ApiEndpoint.disableAddressCache")
            .z()
            .expect("ApiEndpoint.disableAddressCache is not a bool");
        endpoint.disable_tls = env
            .call_method(object, "component4", "()Z", &[])
            .expect("missing ApiEndpoint.disableTls")
            .z()
            .expect("ApiEndpoint.disableTls is not a bool");
    }

    Some(endpoint)
}

/// Resolves the hostname and port to SocketAddr
fn try_socketaddr_from_java(env: &JnixEnv<'_>, endpoint: JObject<'_>) -> Option<SocketAddr> {
    let class = env.get_class("net/mullvad/mullvadvpn/lib/endpoint/ApiEndpoint$Custom");

    let method_id = env
        .get_method_id(&class, "component1", "()Ljava/lang/String;")
        .expect("Failed to get method ID for ApiEndpoint.Custom.hostname()");
    let return_type = JavaType::Object("java/lang/String".to_owned());

    let hostname = env
        .call_method_unchecked(endpoint, method_id, return_type, &[])
        .expect("Failed to call ApiEndpoint.Custom.hostname()")
        .l()
        .expect("Call to ApiEndpoint.Custom.hostname( did not return an object");

    if hostname.is_null() {
        return None;
    }

    let port = env
            .call_method(endpoint, "component2", "()I", &[])
            .expect("missing ApiEndpoint.port")
            .i()
            .expect("ApiEndpoint.port is not a int");

    //Resolve ip address from hostname
    let socket_ip_addr = format!("{}:{}", String::from_java(env, hostname), u16::try_from(port).expect("invalid port"))
                    .to_socket_addrs().unwrap().next();

    socket_ip_addr
}

/// Returns the hostname for an ApiEndpoint
fn try_hostname_from_java(env: &JnixEnv<'_>, endpoint: JObject<'_>) -> Option<String> {
    let class = env.get_class("net/mullvad/mullvadvpn/lib/endpoint/ApiEndpoint$Custom");

    let method_id = env
        .get_method_id(&class, "component1", "()Ljava/lang/String;")
        .expect("Failed to get method ID for ApiEndpoint.Custom.hostname()");
    let return_type = JavaType::Object("java/lang/String".to_owned());

    let hostname = env
        .call_method_unchecked(endpoint, method_id, return_type, &[])
        .expect("Failed to call ApiEndpoint.Custom.hostname()")
        .l()
        .expect("Call to ApiEndpoint.Custom.hostname( did not return an object");

    if hostname.is_null() {
        return None;
    }

    Some(String::from_java(env, hostname))
}
