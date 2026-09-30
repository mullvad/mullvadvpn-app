//! Config definition, see [`Config`].

mod test_locations;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use test_locations::TestLocationList;

use super::VmConfig;

/// Global configuration for the `test-manager`.
///
/// Can be modified using either the setting file, see
/// [`crate::config::io::ConfigFile::get_config_path`] or
/// the `test-manager config` CLI subcommand.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    /// Named API environments, e.g. prod or staging, with settings specific to each.
    #[serde(default)]
    pub environments: BTreeMap<String, EnvironmentConfig>,
}

/// Default `mullvad_host`. This should match the production env.
const DEFAULT_MULLVAD_HOST: &str = "mullvad.net";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentConfig {
    /// Domain name of the environment, e.g. `mullvad.net`. It is prefixed with e.g. "api." and
    /// "ipv4.am.i.".
    #[serde(default = "default_mullvad_host")]
    pub mullvad_host: String,
    /// Account number to use for testing.
    pub account: Option<String>,
    #[serde(default)]
    pub test_locations: TestLocationList,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            mullvad_host: default_mullvad_host(),
            account: None,
            test_locations: TestLocationList::default(),
        }
    }
}

fn default_mullvad_host() -> String {
    DEFAULT_MULLVAD_HOST.to_owned()
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RuntimeOptions {
    pub display: Display,
    pub keep_changes: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum Display {
    #[default]
    None,
    Local,
    Vnc,
}

impl Config {
    pub fn get_vm(&self, name: &str) -> Option<&VmConfig> {
        self.vms.get(name)
    }

    pub fn get_environment(&self, name: &str) -> Option<&EnvironmentConfig> {
        self.environments.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_environment_defaults() {
        let config = r#"
            {
                "vms": {},
                "environments": {
                    "prod": {}
                }
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        let prod = config.get_environment("prod").unwrap();
        assert_eq!(prod.mullvad_host, DEFAULT_MULLVAD_HOST);
        assert!(prod.account.is_none());
        assert!(prod.test_locations.0.is_empty());
        assert!(config.get_environment("staging").is_none());
    }

    #[test]
    fn parse_test_location_not_empty() {
        let config = r#"
            {
                "vms": {},
                "environments": {
                    "staging": {
                        "mullvad_host": "stagemole.eu",
                        "account": "1234123412341234",
                        "test_locations": [
                            { "*daita": [ "se-got-wg-001", "se-got-wg-002" ] },
                            { "*": [ "se" ] }
                        ]
                    }
                }
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        let staging = config.get_environment("staging").unwrap();
        assert_eq!(staging.mullvad_host, "stagemole.eu");
        assert_eq!(staging.account.as_deref(), Some("1234123412341234"));
        assert!(
            staging
                .test_locations
                .lookup("test_daita")
                .unwrap()
                .contains(&"se-got-wg-002".to_string())
        );
    }

    #[test]
    fn parse_legacy_top_level_fields_should_fail() {
        let config = r#"
            {
                "vms": {},
                "mullvad_host": "mullvad.net",
                "test_locations": [ { "*": [ "se" ] } ]
            }"#;

        let _err = serde_json::from_str::<Config>(config).unwrap_err();
    }

    #[test]
    fn parse_multiple_keys_in_map_should_fail() {
        let config = r#"
            {
                "vms": {},
                "environments": {
                    "prod": {
                        "test_locations": [
                            {
                                "*daita": [ "se-got-wg-001", "se-got-wg-002" ],
                                "*test": ["se"]
                            },
                        ]
                    }
                }
            }"#;

        let _err = serde_json::from_str::<Config>(config).unwrap_err();
    }
}
