#[cfg(feature = "api-override")]
use jnix::FromJava;
use jnix::{JnixEnv, jni::objects::JObject};
#[cfg(feature = "api-override")]
use std::net::{IpAddr, ToSocketAddrs};
#[cfg(feature = "api-override")]
use std::str::FromStr;

#[cfg(feature = "api-override")]
pub fn api_endpoint_from_java(
    env: &JnixEnv<'_>,
    endpoint_override: JObject<'_>,
) -> Option<mullvad_api::ApiEndpoint> {
    if endpoint_override.is_null() {
        return None;
    }

    let hostname = hostname_from_java(env, endpoint_override);
    let address = address_from_java(env, endpoint_override);
    let ip_addr = IpAddr::from_str(&address).expect("Invalid IP address");
    let port = port_from_java(env, endpoint_override);
    let socket_addr = (ip_addr, port).to_socket_addrs().unwrap().next().unwrap();
    let sigsum_trusted_pubkeys = sigsum_trusted_pubkeys_from_java(env, endpoint_override);
    let parsed_pubkeys = mullvad_api::ApiEndpoint::parse_sigsum_pubkeys(&sigsum_trusted_pubkeys)
        .expect("invalid sigsum trusted pubkeys");

    Some(mullvad_api::ApiEndpoint {
        host: Some(hostname),
        address: Some(socket_addr),
        sigsum_trusted_pubkeys: Some(parsed_pubkeys),
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
fn address_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> String {
    let hostname = env
        .call_method(endpoint_override, "component2", "()Ljava/lang/String;", &[])
        .expect("missing ApiEndpointOverride.address")
        .l()
        .expect("ApiEndpointOverride.address is not a string");

    String::from_java(env, hostname)
}

#[cfg(feature = "api-override")]
fn port_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> u16 {
    let port = env
        .call_method(endpoint_override, "component3", "()I", &[])
        .expect("missing ApiEndpointOverride.port")
        .i()
        .expect("ApiEndpointOverride.port is not a int");

    u16::try_from(port).expect("invalid port")
}

#[cfg(feature = "api-override")]
fn disable_tls_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> bool {
    env.call_method(endpoint_override, "component4", "()Z", &[])
        .expect("missing ApiEndpointOverride.disableTls")
        .z()
        .expect("ApiEndpointOverride.disableTls is not a bool")
}

#[cfg(feature = "api-override")]
fn force_direct_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> bool {
    env.call_method(endpoint_override, "component5", "()Z", &[])
        .expect("missing ApiEndpointOverride.forceDirectConnection")
        .z()
        .expect("ApiEndpointOverride.forceDirectConnection is not a bool")
}

#[cfg(feature = "api-override")]
fn sigsum_trusted_pubkeys_from_java(env: &JnixEnv<'_>, endpoint_override: JObject<'_>) -> String {
    let pubkeys = env
        .call_method(endpoint_override, "component6", "()Ljava/lang/String;", &[])
        .expect("missing ApiEndpointOverride.sigsumTrustedPubkeys")
        .l()
        .expect("ApiEndpointOverride.forceDirectConnection is not a string");

    String::from_java(env, pubkeys)
}
