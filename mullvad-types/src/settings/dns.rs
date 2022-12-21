#[cfg(target_os = "android")]
use jnix::{FromJava, IntoJava};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum DnsState {
    #[default]
    Default,
    Custom,
}

/// DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct DnsOptions {
    pub state: DnsState,
    pub default_options: DefaultDnsOptions,
    pub custom_options: CustomDnsOptions,
}

/// Default DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct DefaultDnsOptions {
    pub block_ads: bool,
    pub block_trackers: bool,
    pub block_malware: bool,
    pub block_adult_content: bool,
    pub block_gambling: bool,
}

/// Custom DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(target_os = "android", derive(FromJava, IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct CustomDnsOptions {
    pub addresses: Vec<IpAddr>,
}
