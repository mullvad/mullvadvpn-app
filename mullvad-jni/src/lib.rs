use jni::{objects::JObject, JNIEnv};

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_MullvadDaemon_initialize(_: JNIEnv, _: JObject) {
}
