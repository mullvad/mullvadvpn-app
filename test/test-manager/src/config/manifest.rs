//! Config definition, see [`Config`].

mod test_locations;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use test_locations::TestLocationList;

use super::VmConfig;
use crate::tests::config::DEFAULT_MULLVAD_HOST;

/// Global configuration for the `test-manager`.
///
/// Can be modified using either the setting file, see
/// [`crate::config::io::ConfigFile::get_config_path`] or
/// the `test-manager config` CLI subcommand.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    pub mullvad_host: Option<String>,
    #[serde(default)]
    pub test_locations: TestLocationList,
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

    /// Get the Mullvad host to use.
    ///
    /// Defaults to [`DEFAULT_MULLVAD_HOST`] if the host was not provided in the [`ConfigFile`].
    pub fn get_host(&self) -> String {
        self.mullvad_host.clone().unwrap_or_else(|| {
            log::debug!("No Mullvad host has been set explicitly. Falling back to default host");
            DEFAULT_MULLVAD_HOST.to_owned()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test_location_empty() {
        let config = r#"
            {
                "vms": {},
                "mullvad_host": "mullvad.net"
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        assert!(config.test_locations.0.is_empty());
    }

    #[test]
    fn parse_test_location_not_empty() {
        let config = r#"
            {
                "vms": {},
                "mullvad_host": "mullvad.net",
                "test_locations": [
                    { "*daita": [ "se-got-wg-001", "se-got-wg-002" ] },
                    { "*": [ "se" ] }
                ]
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        assert!(config
            .test_locations
            .lookup("test_daita")
            .unwrap()
            .contains(&"se-got-wg-002".to_string()));
        assert!(!config.test_locations.0.is_empty());
    }
}
