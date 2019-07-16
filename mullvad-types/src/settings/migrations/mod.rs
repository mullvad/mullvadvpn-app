use super::{Error, Result, Settings};
use std::io::{Read, Seek, SeekFrom};
mod v1;


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


trait SettingsVersion {
    fn read(&self, reader: &mut dyn Read) -> Result<VersionedSettings>;
    fn migrate(&self, settings: VersionedSettings) -> VersionedSettings;
}

fn migrations() -> Vec<Box<dyn SettingsVersion>> {
    vec![Box::new(v1::Migration)]
}

pub fn try_migrate_settings<R: Read + Seek>(
    mut reader: &mut R,
) -> Result<crate::settings::Settings> {
    let mut migrations_to_apply = vec![];
    let mut valid_settings = None;

    let migrations = migrations();
    for migration in migrations.iter().rev() {
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|e| Error::ReadError(format!("Failed to seek reader"), e))?;
        match migration.read(&mut reader) {
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
