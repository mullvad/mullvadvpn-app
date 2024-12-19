#[cfg(feature = "api-override")]
use jnix::FromJava;
use jnix::{jni::objects::JObject, JnixEnv};
#[cfg(feature = "api-override")]
use std::net::{SocketAddr, ToSocketAddrs};

#[cfg(feature = "api-override")]
pub fn api_endpoint_from_java(
    env: &JnixEnv<'_>,
    endpoint_override: JObject<'_>,
) -> Option<mullvad_api::ApiEndpoint> {
    if endpoint_override.is_null() {
        return None;
    }

    let hostname = hostname_from_java(env, endpoint_override);
    let port = port_from_java(env, endpoint_override);
    let address = Some(create_socket_addr(hostname.clone(), port));

    Some(mullvad_api::ApiEndpoint {
        host: Some(hostname),
        address,
        disable_tls: disable_tls_from_java(env, endpoint_override),
        force_direct: force_direct_from_java(env, endpoint_override),
    })
}

#[cfg(not(feature = "api-override"))]
pub fn api_endpoint_from_java(
    _env: &JnixEnv<'_>,
    endpoint_override: JObject<'_>,
) -> Option<mullvad_api::ApiEndpoint> {
    if endpoint_override.is_null() {
        return None;
    }
    panic!("Trying to set api override when feature is disabled")
}

/// Resolves the hostname and port to SocketAddr
#[cfg(feature = "api-override")]
fn create_socket_addr(hostname: String, port: u16) -> SocketAddr {
    //Resolve ip address from hostname
    (hostname, port)
        .to_socket_addrs()
        .expect("could not resolve address")
        .next()
        .expect("no ip address received")
}

#[cfg(feature = "api-override")]
fn hostname_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> String {
    let hostname = env
        .call_method(endpoint_override, "component1", "()Ljava/lang/String;", &[])
        .expect("missing ApiEndpointOverride.hostname")
        .l()
        .expect("ApiEndpointOverride.hostname is not a string");

    String::from_java(env, hostname)
}

#[cfg(feature = "api-override")]
fn port_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> u16 {
    let port = env
        .call_method(endpoint_override, "component2", "()I", &[])
        .expect("missing ApiEndpointOverride.port")
        .i()
        .expect("ApiEndpointOverride.port is not a int");

    u16::try_from(port).expect("invalid port")
}

#[cfg(feature = "api-override")]
fn disable_tls_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> bool {
    env.call_method(endpoint_override, "component3", "()Z", &[])
        .expect("missing ApiEndpointOverride.disableTls")
        .z()
        .expect("ApiEndpointOverride.disableTls is not a bool")
}

#[cfg(feature = "api-override")]
fn force_direct_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> bool {
    env.call_method(endpoint_override, "component4", "()Z", &[])
        .expect("missing ApiEndpointOverride.forceDirectConnection")
        .z()
        .expect("ApiEndpointOverride.forceDirectConnection is not a bool")
}
