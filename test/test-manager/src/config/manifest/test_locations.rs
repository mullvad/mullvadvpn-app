use serde::{
    de::{Deserialize, Deserializer, Error, MapAccess, Visitor},
    ser::{Serialize, SerializeMap},
    Deserialize as DeserDerive, Serialize as SerDerive,
};
use std::fmt;

#[derive(Clone, Default)]
pub struct TestLocation(glob::Pattern, Vec<String>);

impl fmt::Debug for TestLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.0, &self.1)
    }
}

/// Relay/location overrides for tests.
///
/// # Deserializing with `serde-json`
///
/// The format is a list of maps with a single key-value
/// pair, where the key is a glob pattern that will be matched against the test name, and the
/// value is a list of locations to use for that test. The first match will be used.
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
/// The above example will set the locations for the test `test_daita` to a custom list
/// containing `se-got-wg-001` and `se-got-wg-002`. The `*` is a wildcard that will match
/// any test name. The order of the list is important, as the first match will be used.
#[derive(Debug, DeserDerive, SerDerive, Clone, Default)]
pub struct TestLocationList(pub Vec<TestLocation>);

impl TestLocationList {
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
        let (key, value) = access
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
