#[cfg(target_os = "android")]
use jnix::{jni::objects::JObject, FromJava, IntoJava, JnixEnv};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// When we want to block certain contents with the help of DNS server side,
/// we compute the resolver IP to use based on these constants. The last
/// byte can be ORed together to combine multiple block lists.
const DNS_BLOCKING_IP_BASE: Ipv4Addr = Ipv4Addr::new(100, 64, 0, 0);
const DNS_AD_BLOCKING_IP_BIT: u8 = 1 << 0; // 0b00000001
const DNS_TRACKER_BLOCKING_IP_BIT: u8 = 1 << 1; // 0b00000010
const DNS_MALWARE_BLOCKING_IP_BIT: u8 = 1 << 2; // 0b00000100
const DNS_ADULT_BLOCKING_IP_BIT: u8 = 1 << 3; // 0b00001000
const DNS_GAMBLING_BLOCKING_IP_BIT: u8 = 1 << 4; // 0b00010000

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

impl DnsOptions {
    /// Return the resolvers as a vector of `IpAddr`s. Returns `None` when no special resolvers
    /// are requested and the tunnel default gateway should be used.
    pub fn to_addresses(&self) -> Option<Vec<IpAddr>> {
        match self.state {
            DnsState::Default => {
                // Check if we should use a custom blocking DNS resolver.
                // And if so, compute the IP.
                let mut last_byte: u8 = 0;

                if self.default_options.block_ads {
                    last_byte |= DNS_AD_BLOCKING_IP_BIT;
                }
                if self.default_options.block_trackers {
                    last_byte |= DNS_TRACKER_BLOCKING_IP_BIT;
                }
                if self.default_options.block_malware {
                    last_byte |= DNS_MALWARE_BLOCKING_IP_BIT;
                }
                if self.default_options.block_adult_content {
                    last_byte |= DNS_ADULT_BLOCKING_IP_BIT;
                }
                if self.default_options.block_gambling {
                    last_byte |= DNS_GAMBLING_BLOCKING_IP_BIT;
                }

                if last_byte != 0 {
                    let mut dns_ip = DNS_BLOCKING_IP_BASE.octets();
                    dns_ip[dns_ip.len() - 1] |= last_byte;
                    Some(vec![IpAddr::V4(Ipv4Addr::from(dns_ip))])
                } else {
                    None
                }
            }
            DnsState::Custom => {
                if self.custom_options.addresses.is_empty() {
                    None
                } else {
                    Some(self.custom_options.addresses.clone())
                }
            }
        }
    }
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
