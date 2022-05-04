#[cfg(target_os = "android")]
use jnix::{jni::objects::JObject, FromJava, IntoJava, JnixEnv};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DnsState {
    Default,
    Custom,
}

impl Default for DnsState {
    fn default() -> Self {
        Self::Default
    }
}

/// DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct DnsOptions {
    #[cfg_attr(target_os = "android", jnix(map = "|state| state == DnsState::Custom"))]
    pub state: DnsState,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub default_options: DefaultDnsOptions,
    #[cfg_attr(target_os = "android", jnix(map = "|opts| opts.addresses"))]
    pub custom_options: CustomDnsOptions,
}

#[cfg(target_os = "android")]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(target_os = "android", derive(FromJava))]
#[cfg_attr(
    target_os = "android",
    jnix(class_name = "net.mullvad.mullvadvpn.model.DnsOptions")
)]
pub struct AndroidDnsOptions {
    pub custom: bool,
    pub addresses: Vec<IpAddr>,
}

#[cfg(target_os = "android")]
impl From<AndroidDnsOptions> for DnsOptions {
    fn from(options: AndroidDnsOptions) -> Self {
        Self {
            state: if options.custom {
                DnsState::Custom
            } else {
                DnsState::Default
            },
            default_options: DefaultDnsOptions::default(),
            custom_options: CustomDnsOptions {
                addresses: options.addresses,
            },
        }
    }
}

#[cfg(target_os = "android")]
impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for DnsOptions
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Lnet/mullvad/mullvadvpn/model/DnsOptions";

    fn from_java(env: &JnixEnv<'env>, object: JObject<'sub_env>) -> Self {
        AndroidDnsOptions::from_java(env, object).into()
    }
}

/// Default DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct DefaultDnsOptions {
    pub block_ads: bool,
    pub block_trackers: bool,
    pub block_malware: bool,
    pub block_adult_content: bool,
    pub block_gambling: bool,
}

/// Custom DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomDnsOptions {
    pub addresses: Vec<IpAddr>,
}
