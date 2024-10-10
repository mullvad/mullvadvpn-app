use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

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
}

/// Custom DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomDnsOptions {
    pub addresses: Vec<IpAddr>,
}

// ======================================================

pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V4");

    let dns_options =
        || -> Option<&serde_json::Value> { settings.get("tunnel_options")?.get("dns_options") }();

    if let Some(options) = dns_options
        && options.get("state").is_none()
    {
        let new_state = if options
            .get("custom")
            .map(|custom| custom.as_bool().unwrap_or(false))
            .unwrap_or(false)
        {
            DnsState::Custom
        } else {
            DnsState::Default
        };
        let addresses = if let Some(addrs) = options.get("addresses") {
            serde_json::from_value(addrs.clone()).map_err(|_| Error::InvalidSettingsContent)?
        } else {
            vec![]
        };

        settings["tunnel_options"]["dns_options"] = serde_json::json!(DnsOptions {
            state: new_state,
            default_options: DefaultDnsOptions::default(),
            custom_options: CustomDnsOptions { addresses },
        });
    }

    settings["settings_version"] = serde_json::json!(SettingsVersion::V4);

    Ok(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V3 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use crate::migrations::load_seed;

    #[test]
    fn v3_to_v4_migration() {
        let mut settings = load_seed("v3.json");
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings).unwrap());
    }
}
