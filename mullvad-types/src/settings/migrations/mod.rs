use super::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
mod v1;


#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u32)]
pub enum SettingsVersion {
    V2 = 2,
}

pub const CURRENT_SETTINGS_VERSION: SettingsVersion = SettingsVersion::V2;

impl<'de> Deserialize<'de> for SettingsVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match <u32>::deserialize(deserializer)? {
            v if v == SettingsVersion::V2 as u32 => Ok(SettingsVersion::V2),
            v => Err(serde::de::Error::custom(format!(
                "{} is not a valid SettingsVersion",
                v
            ))),
        }
    }
}

impl Serialize for SettingsVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(*self as u32)
    }
}


trait SettingsMigration {
    fn version_matches(&self, settings: &mut serde_json::Value) -> bool;
    fn migrate(&self, settings: &mut serde_json::Value) -> Result<()>;
}

pub fn try_migrate_settings(mut settings_file: &[u8]) -> Result<crate::settings::Settings> {
    let mut settings: serde_json::Value =
        serde_json::from_reader(&mut settings_file).map_err(Error::ParseError)?;

    if !settings.is_object() {
        return Err(Error::NoMatchingVersion);
    }

    for migration in &[Box::new(v1::Migration)] {
        if !migration.version_matches(&mut settings) {
            continue;
        }
        migration.migrate(&mut settings)?;
    }

    serde_json::from_value(settings).map_err(Error::ParseError)
}

#[cfg(test)]
mod test {
    use super::SettingsVersion;
    use serde_json;

    #[test]
    #[should_panic]
    fn test_deserialization_failure_version_too_small() {
        let _version: SettingsVersion = serde_json::from_str("1").expect("Version too small");
    }

    #[test]
    #[should_panic]
    fn test_deserialization_failure_version_too_big() {
        let _version: SettingsVersion = serde_json::from_str("3").expect("Version too big");
    }

    #[test]
    fn test_deserialization_success() {
        let _version: SettingsVersion =
            serde_json::from_str("2").expect("Failed to deserialize valid version");
    }

    #[test]
    fn test_serialization_success() {
        let version = SettingsVersion::V2;
        let s = serde_json::to_string(&version).expect("Failed to serialize");
        assert_eq!(s, "2");
    }
}
