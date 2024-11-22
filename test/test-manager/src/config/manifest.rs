//! Config definition.
//! TODO: Document struct and link to that documentation

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::VmConfig;
use crate::tests::config::DEFAULT_MULLVAD_HOST;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    pub mullvad_host: Option<String>,
    /// Add location override on a per-test basis. These those locations will be the
    /// only available options for the given test to pick from!
    ///
    /// Glob patterns are used to targeet one or more tests with a set of locations. If there are multiple
    /// patterns that match with a test name, only the first match will be considered. The ordering
    /// is just like a regular JSON list.
    // TODO: Make sure this is not serialized into null
    pub location: Option<locations::Locations>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct RuntimeOptions {
    pub display: Display,
    pub keep_changes: bool,
}

#[derive(Default, Serialize, Deserialize, Clone)]
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

mod locations {
    use std::ops::Deref;

    use serde::{de::Visitor, Deserialize, Serialize};

    // "location": {
    //   "override": [
    //     { "test": "test_daita", locations: ["se-got-101"] },
    //     { "test": "*", locations: ["Nordic"] }
    //   ]
    // },

    #[derive(Serialize, Deserialize, Clone, Default)]
    pub struct Locations {
        pub r#override: Overrides,
    }

    impl Locations {
        // Look up a test (by name) and see if there are any locations
        // that we should use.
        pub fn lookup(&self, test: &str) -> Option<&Vec<String>> {
            self.r#override.lookup(test)
        }
    }

    /// Mapping of glob pattern to a set of locations.
    #[derive(Serialize, Deserialize, Clone)]
    pub struct Overrides(Vec<Override>);

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Override {
        test: SerializeableGlob,
        locations: Vec<String>,
    }

    impl Overrides {
        // Lookup the first test that matches a glob pattern.
        fn lookup(&self, test: &str) -> Option<&Vec<String>> {
            self.0
                .iter()
                .find(
                    |Override {
                         test: test_glob, ..
                     }| test_glob.matches(test),
                )
                .map(|Override { locations, .. }| locations)
        }
    }

    impl Default for Overrides {
        /// All tests default to using the "any" location.
        /// Written out in a config it would look like the following: { "*": ["any"] }
        fn default() -> Self {
            let overrides = {
                let glob = SerializeableGlob::from(glob::Pattern::new("*").unwrap());
                vec![Override {
                    test: glob,
                    locations: vec!["any".to_string()],
                }]
            };
            Overrides(overrides)
        }
    }

    /// Implement serde [Serialize] and [Deserialize] for [glob::Pattern].
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
    struct SerializeableGlob(glob::Pattern);

    impl<'de> Deserialize<'de> for SerializeableGlob {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let glob = deserializer.deserialize_string(GlobVisitor)?;
            Ok(SerializeableGlob(glob))
        }
    }

    impl Serialize for SerializeableGlob {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let glob = self.0.as_str();
            serializer.serialize_str(glob)
        }
    }

    impl From<glob::Pattern> for SerializeableGlob {
        fn from(pattern: glob::Pattern) -> Self {
            Self(pattern)
        }
    }

    impl Deref for SerializeableGlob {
        type Target = glob::Pattern;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    struct GlobVisitor;

    impl Visitor<'_> for GlobVisitor {
        type Value = glob::Pattern;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("Only strings can be deserialised to glob pattern")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            glob::Pattern::new(v).map_err(|err| {
                E::custom(format!(
                    "Cannot compile glob pattern from: {v} error: {err:?}"
                ))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_relay_location_per_test_override() {
        let config = "
            {
                \"vms\": {},
                \"mullvad_host\": \"mullvad.net\",
                \"location\": { \"override\": { \"*\": [\"Low Latency\"] } }
            }";

        let config: Config = serde_json::from_str(config).unwrap();
        let _location = config.location.expect("location overrides was not parsed");
    }
}
