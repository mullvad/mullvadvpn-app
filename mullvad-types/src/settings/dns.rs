use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DnsState {
    #[default]
    Default,
    Custom,
}

/// DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct DnsOptions {
    pub state: DnsState,
    pub default_options: DefaultDnsOptions,
    pub custom_options: CustomDnsOptions,
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
    pub block_social_media: bool,
}

/// Custom DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomDnsOptions {
    pub addresses: Vec<IpAddr>,
}

impl DefaultDnsOptions {
    /// Return whether any content blockers are enabled.
    pub fn any_blockers_enabled(&self) -> bool {
        let DefaultDnsOptions {
            block_ads,
            block_trackers,
            block_malware,
            block_adult_content,
            block_gambling,
            block_social_media,
        } = *self;

        block_ads
            || block_trackers
            || block_malware
            || block_adult_content
            || block_gambling
            || block_social_media
    }
}
