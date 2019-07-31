use super::{Error, Result, Settings};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io::Read;
mod v1;


#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SettingsVersion {
    V2 = 2,
}

impl SettingsVersion {
    pub fn as_u32(&self) -> u32 {
        unsafe { ::std::mem::transmute(*self) }
    }

    pub fn max_version() -> Self {
        SettingsVersion::V2
    }

    pub fn min_version() -> Self {
        SettingsVersion::V2
    }
}

impl<'de> Deserialize<'de> for SettingsVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = <u32>::deserialize(deserializer)?;
        if version < SettingsVersion::min_version().as_u32() {
            return Err(serde::de::Error::custom(format!(
                "Version number {} too small",
                version
            )));
        }

        if version > SettingsVersion::max_version().as_u32() {
            return Err(serde::de::Error::custom(format!(
                "Version number {} too large",
                version
            )));
        }

        unsafe { Ok(::std::mem::transmute(version)) }
    }
}

impl Serialize for SettingsVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.as_u32())
    }
}


#[derive(Debug)]
enum VersionedSettings {
    V1(v1::Settings),
    V2(crate::settings::Settings),
}

impl VersionedSettings {
    /// Unrwaps the latest version of settings or panics.
    fn unwrap(self) -> Settings {
        match self {
            VersionedSettings::V2(settings) => settings,
            lower => {
                panic!("Unexpected settings version - {:?}", lower);
            }
        }
    }
}


trait SettingsMigration {
    fn read(&self, reader: &mut dyn Read) -> Result<VersionedSettings>;
    fn migrate(&self, settings: VersionedSettings) -> VersionedSettings;
}

fn migrations() -> Vec<Box<dyn SettingsMigration>> {
    vec![Box::new(v1::Migration)]
}

pub fn try_migrate_settings(mut settings_file: &[u8]) -> Result<crate::settings::Settings> {
    let mut migrations_to_apply = vec![];
    let mut valid_settings = None;

    let migrations = migrations();
    for migration in migrations.iter() {
        match migration.read(&mut settings_file) {
            Ok(settings) => {
                valid_settings = Some(migration.migrate(settings));
                break;
            }
            Err(_e) => {
                migrations_to_apply.push(migration);
            }
        };
    }

    if let Some(settings) = valid_settings {
        let upgraded_settings = migrations_to_apply
            .iter()
            .rev()
            .fold(settings, |old_settings, migration| {
                migration.migrate(old_settings)
            });
        return Ok(upgraded_settings.unwrap());
    }
    return Err(Error::NoMatchingVersion);
}

#[cfg(test)]
mod test {
    use super::SettingsVersion;

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
