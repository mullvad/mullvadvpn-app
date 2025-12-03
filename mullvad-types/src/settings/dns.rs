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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct DefaultDnsOptions {
    pub block_ads: bool,
    pub block_trackers: bool,
    pub block_malware: bool,
    pub block_adult_content: bool,
    pub block_gambling: bool,
    pub block_social_media: bool,
}

impl DefaultDnsOptions {
    /// Create [`DefaultDnsOptions`] which does not configure DNS.
    pub const fn new() -> Self {
        Self {
            block_ads: false,
            block_trackers: false,
            block_malware: false,
            block_adult_content: false,
            block_gambling: false,
            block_social_media: false,
        }
    }

    /// Enable ads DNS content blocker.
    pub const fn block_ads(mut self) -> Self {
        self.block_ads = true;
        self
    }

    /// Enable trackers DNS content blocker.
    pub const fn block_trackers(mut self) -> Self {
        self.block_trackers = true;
        self
    }

    /// Enable malware DNS content blocker.
    pub const fn block_malware(mut self) -> Self {
        self.block_malware = true;
        self
    }

    /// Enable adult content DNS content blocker.
    pub const fn block_adult_content(mut self) -> Self {
        self.block_adult_content = true;
        self
    }

    /// Enable gambling DNS content blocker.
    pub const fn block_gambling(mut self) -> Self {
        self.block_gambling = true;
        self
    }

    /// Enable social media DNS content blocker.
    pub const fn block_social_media(mut self) -> Self {
        self.block_social_media = true;
        self
    }

    /// Return whether any content blockers are enabled.
    pub const fn any_blockers_enabled(&self) -> bool {
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

impl Default for DefaultDnsOptions {
    fn default() -> DefaultDnsOptions {
        DefaultDnsOptions::new()
    }
}

/// Custom DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomDnsOptions {
    pub addresses: Vec<IpAddr>,
}
