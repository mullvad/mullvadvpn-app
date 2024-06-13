use jnix::{
    jni::{
        objects::JObject,
        signature::{JavaType, Primitive},
    },
    FromJava, JnixEnv,
};
use std::net::{IpAddr, SocketAddr};

pub fn api_endpoint_from_java(
    env: &JnixEnv<'_>,
    object: JObject<'_>,
) -> Option<mullvad_api::ApiEndpoint> {
    if object.is_null() {
        return None;
    }

    let mut endpoint = mullvad_api::ApiEndpoint::from_env_vars();

    let address = env
        .call_method(object, "component1", "()Ljava/net/InetSocketAddress;", &[])
        .expect("missing ApiEndpoint.address")
        .l()
        .expect("ApiEndpoint.address is not an InetSocketAddress");

    endpoint.address = Some(
        try_socketaddr_from_java(env, address).expect("received unresolved InetSocketAddress"),
    );
    endpoint.host = try_hostname_from_java(env, address);
    #[cfg(feature = "api-override")]
    {
        endpoint.disable_address_cache = env
            .call_method(object, "component2", "()Z", &[])
            .expect("missing ApiEndpoint.disableAddressCache")
            .z()
            .expect("ApiEndpoint.disableAddressCache is not a bool");
        endpoint.disable_tls = env
            .call_method(object, "component3", "()Z", &[])
            .expect("missing ApiEndpoint.disableTls")
            .z()
            .expect("ApiEndpoint.disableTls is not a bool");
    }

    Some(endpoint)
}

/// Converts InetSocketAddress to a SocketAddr. Return `None` if the hostname is unresolved.
fn try_socketaddr_from_java(env: &JnixEnv<'_>, address: JObject<'_>) -> Option<SocketAddr> {
    let class = env.get_class("java/net/InetSocketAddress");

    let method_id = env
        .get_method_id(&class, "getAddress", "()Ljava/net/InetAddress;")
        .expect("Failed to get method ID for InetSocketAddress.getAddress()");
    let return_type = JavaType::Object("java/net/InetAddress".to_owned());

    let ip_addr = env
        .call_method_unchecked(address, method_id, return_type, &[])
        .expect("Failed to call InetSocketAddress.getAddress()")
        .l()
        .expect("Call to InetSocketAddress.getAddress() did not return an object");

    if ip_addr.is_null() {
        return None;
    }

    let method_id = env
        .get_method_id(&class, "getPort", "()I")
        .expect("Failed to get method ID for InetSocketAddress.getPort()");
    let return_type = JavaType::Primitive(Primitive::Int);

    let port = env
        .call_method_unchecked(address, method_id, return_type, &[])
        .expect("Failed to call InetSocketAddress.getPort()")
        .i()
        .expect("Call to InetSocketAddress.getPort() did not return an int");

    Some(SocketAddr::new(
        IpAddr::from_java(env, ip_addr),
        u16::try_from(port).expect("invalid port"),
    ))
}

/// Returns the hostname for an InetSocketAddress. This may be an IP address converted to a string.
fn try_hostname_from_java(env: &JnixEnv<'_>, address: JObject<'_>) -> Option<String> {
    let class = env.get_class("java/net/InetSocketAddress");

    let method_id = env
        .get_method_id(&class, "getHostString", "()Ljava/lang/String;")
        .expect("Failed to get method ID for InetSocketAddress.getHostString()");
    let return_type = JavaType::Object("java/lang/String".to_owned());

    let hostname = env
        .call_method_unchecked(address, method_id, return_type, &[])
        .expect("Failed to call InetSocketAddress.getHostString()")
        .l()
        .expect("Call to InetSocketAddress.getHostString() did not return an object");

    if hostname.is_null() {
        return None;
    }

    Some(String::from_java(env, hostname))
}
