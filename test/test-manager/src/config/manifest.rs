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

/// Built-in environments, e.g. those used by the GitHub workflow. Environments in the config file
/// override these field by field.
const BUILTIN_ENVIRONMENTS: &str = include_str!("../../../environments.json");

/// Settings for an environment. Unset fields fall back on the built-in environment of the same
/// name, if any, and then on defaults.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentConfig {
    /// Domain name of the environment, e.g. `mullvad.net`. It is prefixed with e.g. "api." and
    /// "ipv4.am.i.". Defaults to [DEFAULT_MULLVAD_HOST].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mullvad_host: Option<String>,
    /// Account number to use for testing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_locations: Option<TestLocationList>,
}

impl EnvironmentConfig {
    pub fn mullvad_host(&self) -> &str {
        self.mullvad_host.as_deref().unwrap_or(DEFAULT_MULLVAD_HOST)
    }

    /// Return the locations configured for `test`, if any.
    pub fn test_locations(&self, test: &str) -> Option<&Vec<String>> {
        self.test_locations.as_ref()?.lookup(test)
    }

    /// Return `self`, with the fields that are set in `overrides` replaced.
    fn merge(self, overrides: EnvironmentConfig) -> EnvironmentConfig {
        EnvironmentConfig {
            mullvad_host: overrides.mullvad_host.or(self.mullvad_host),
            account: overrides.account.or(self.account),
            test_locations: overrides.test_locations.or(self.test_locations),
        }
    }
}

fn builtin_environments() -> BTreeMap<String, EnvironmentConfig> {
    serde_json::from_str(BUILTIN_ENVIRONMENTS).expect("built-in environments must be valid")
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

    /// Return the environment `name`, combining the built-in environment and the one in the
    /// config file. Returns `None` if neither exists.
    pub fn get_environment(&self, name: &str) -> Option<EnvironmentConfig> {
        let builtin = builtin_environments().remove(name);
        let configured = self.environments.get(name).cloned();
        match (builtin, configured) {
            (Some(builtin), Some(configured)) => Some(builtin.merge(configured)),
            (builtin, configured) => builtin.or(configured),
        }
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
                    "custom": {}
                }
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        let custom = config.get_environment("custom").unwrap();
        assert_eq!(custom.mullvad_host(), DEFAULT_MULLVAD_HOST);
        assert!(custom.account.is_none());
        assert!(custom.test_locations.is_none());
        assert!(config.get_environment("nonexistent").is_none());
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
        assert_eq!(staging.mullvad_host(), "stagemole.eu");
        assert_eq!(staging.account.as_deref(), Some("1234123412341234"));
        assert!(
            staging
                .test_locations("test_daita")
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

    #[test]
    fn parse_builtin_environments() {
        let environments = builtin_environments();
        assert!(environments.contains_key("prod"));
        assert!(environments.contains_key("staging"));
    }

    #[test]
    fn config_overrides_builtin_environment_by_field() {
        let config = r#"
            {
                "vms": {},
                "environments": {
                    "staging": { "account": "1234123412341234" },
                    "prod": { "test_locations": [ { "*": [ "ch" ] } ] }
                }
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        let builtin = builtin_environments();

        let staging = config.get_environment("staging").unwrap();
        assert_eq!(staging.account.as_deref(), Some("1234123412341234"));
        assert_eq!(staging.mullvad_host(), builtin["staging"].mullvad_host());

        let prod = config.get_environment("prod").unwrap();
        assert_eq!(
            prod.test_locations("test_anything"),
            Some(&vec!["ch".to_string()])
        );
        assert_eq!(prod.mullvad_host(), builtin["prod"].mullvad_host());
    }
}
