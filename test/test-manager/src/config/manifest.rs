//! Config definition.
//! TODO: Document struct and link to that documentation

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::VmConfig;
use crate::tests::config::DEFAULT_MULLVAD_HOST;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    pub mullvad_host: Option<String>,
    /// Relay/location overrides for tests. The format is a list of maps, where the key is a glob
    /// pattern that will be matched against the test name, and the value is a list of locations to
    /// use for that test. The first match will be used.
    ///
    /// Example:
    /// ```json
    /// {
    ///   // other fields
    ///    "test_locations": [
    ///        { "*daita*": [ "se-got-wg-001", "se-got-wg-002" ] },
    ///        { "*": [ "se" ] }
    ///    ]
    /// }
    /// ```
    ///
    /// The above example will set the locations for the test `test_daita` to  a custom list
    /// containing `se-got-wg-001` and `se-got-wg-002`. The `*` is a wildcard that will match
    /// any test name. The order of the list is important, as the first match will be used.
    #[serde(default)]
    pub test_locations: locations::TestLocationList,
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

mod locations {
    use serde::{
        de::{Deserialize, Deserializer, Error, MapAccess, Visitor},
        ser::{Serialize, SerializeMap},
        Deserialize as DeserDerive, Serialize as SerDerive,
    };
    use std::fmt;

    #[derive(Debug, Clone, Default)]
    pub struct TestLocation(glob::Pattern, Vec<String>);

    #[derive(Debug, DeserDerive, SerDerive, Clone, Default)]
    pub struct TestLocationList(pub Vec<TestLocation>);

    impl TestLocationList {
        // TODO: Consider if we should handle the case of an empty list by returning vec!["any"]
        /// Look up a test (by name) and see if there are any locations
        /// that we should use.
        pub fn lookup(&self, test: &str) -> Option<&Vec<String>> {
            self.0
                .iter()
                .find(|TestLocation(test_glob, _)| test_glob.matches(test))
                .map(|TestLocation(_, locations)| locations)
        }
    }

    struct TestLocationVisitor;

    impl<'de> Visitor<'de> for TestLocationVisitor {
        // The type that our Visitor is going to produce.
        type Value = TestLocation;

        // Format a message stating what data this Visitor expects to receive.
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("A list of maps")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let (key, value) =
                access
                    .next_entry::<String, Vec<String>>()?
                    .ok_or(M::Error::custom(
                    "Test location map should contain exactly one key-value pair, but it was empty",
                ))?;
            let glob = glob::Pattern::new(&key).map_err(|err| {
                M::Error::custom(format!(
                    "Cannot compile glob pattern from: {key} error: {err:?}"
                ))
            })?;

            if let Some((key, value)) = access.next_entry::<String, Vec<String>>()? {
                return Err(M::Error::custom(format!(
                    "Test location map should contain exactly one key-value pair, but found another key: '{key}' and value: '{value:?}'"
                )));
            }

            Ok(TestLocation(glob, value))
        }
    }

    impl<'de> Deserialize<'de> for TestLocation {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(TestLocationVisitor)
        }
    }

    impl Serialize for TestLocation {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry(self.0.as_str(), &self.1)?;
            map.end()
        }
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
                    { "daita": [ "se-got-wg-001", "se-got-wg-002" ] },
                    { "*": [ "se" ] }
                ]
            }"#;

        let config: Config = serde_json::from_str(config).unwrap();
        assert!(!config.test_locations.0.is_empty());
    }
}
