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
    /// Settings specific to each [Environment].
    #[serde(default)]
    pub environments: Environments,
}

/// An environment to test against, i.e. its API and relays.
#[derive(clap::ValueEnum, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Prod,
    Staging,
}

/// Settings for each [Environment].
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct Environments {
    pub prod: EnvironmentConfig,
    pub staging: EnvironmentConfig,
}

impl Environments {
    fn get(&self, env: Environment) -> &EnvironmentConfig {
        match env {
            Environment::Prod => &self.prod,
            Environment::Staging => &self.staging,
        }
    }
}

/// Default `mullvad_host`. This should match the production env.
const DEFAULT_MULLVAD_HOST: &str = "mullvad.net";

/// Built-in environments, e.g. those used by the GitHub workflow. Environments in the config file
/// override these field by field.
const BUILTIN_ENVIRONMENTS: &str = include_str!("../../../environments.json");

/// Settings for an environment. Unset fields fall back on the built-in environment, and then on
/// defaults.
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

fn builtin_environments() -> Environments {
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

    /// Return the settings for `env`, combining the built-in environment and the one in the
    /// config file.
    pub fn get_environment(&self, env: Environment) -> EnvironmentConfig {
        let builtin = builtin_environments().get(env).clone();
        builtin.merge(self.environments.get(env).clone())
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
        let prod = &config.environments.prod;
        assert_eq!(prod.mullvad_host(), DEFAULT_MULLVAD_HOST);
        assert!(prod.account.is_none());
        assert!(prod.test_locations.is_none());
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
        let staging = config.get_environment(Environment::Staging);
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
        assert!(environments.prod.test_locations.is_some());
        assert!(environments.staging.test_locations.is_some());
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

        let staging = config.get_environment(Environment::Staging);
        assert_eq!(staging.account.as_deref(), Some("1234123412341234"));
        assert_eq!(staging.mullvad_host(), builtin.staging.mullvad_host());

        let prod = config.get_environment(Environment::Prod);
        assert_eq!(
            prod.test_locations("test_anything"),
            Some(&vec!["ch".to_string()])
        );
        assert_eq!(prod.mullvad_host(), builtin.prod.mullvad_host());
    }
}
