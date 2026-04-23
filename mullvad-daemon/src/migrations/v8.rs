use super::Result;
use mullvad_types::settings::SettingsVersion;

// This migration doesn't vendor any types.

/// This is a closed migraton.
///
/// If `ofuscation_settings.selected_obfuscation` is `off`, set it to `auto`.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V9");

    migrate_selected_obfuscaton(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V9);

    Ok(())
}

fn migrate_selected_obfuscaton(settings: &mut serde_json::Value) -> Result<()> {
    let Some(selected_obfuscation) = settings
        .get_mut("obfuscation_settings")
        .and_then(|obfuscation_settings| obfuscation_settings.get_mut("selected_obfuscation"))
    else {
        return Ok(());
    };

    if selected_obfuscation == "off" {
        *selected_obfuscation = "auto".into();
    }

    Ok(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V8 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use crate::migrations::load_seed;

    use super::{migrate, migrate_selected_obfuscaton};

    /// Parse example v8 settings as a pretty printed JSON string.
    fn v8_settings() -> serde_json::Value {
        load_seed("v8.json")
    }

    #[test]
    fn snapshot_v8_settings() {
        let v8 = serde_json::to_string_pretty(&v8_settings()).unwrap();
        insta::assert_snapshot!(v8);
    }

    #[test]
    fn test_v8_to_v9_migration() {
        let mut v8 = v8_settings();
        migrate(&mut v8).unwrap();
        let v9 = serde_json::to_string_pretty(&v8).unwrap();
        insta::assert_snapshot!(v9);
    }

    /// For obfuscation_settings
    /// obfuscation_settings: { selected_obfuscation: "on" } should be not be changed.
    #[test]
    fn migrate_seleted_obfuscation_from_on() {
        let mut migrated_settings: serde_json::Value =
            serde_json::from_str(r#"{ "obfuscation_settings": { "selected_obfuscation": "on" } }"#)
                .unwrap();
        let expected_settings = migrated_settings.clone();

        migrate_selected_obfuscaton(&mut migrated_settings).unwrap();

        assert_eq!(migrated_settings, expected_settings);
    }

    /// For obfuscation_settings
    /// obfuscation_settings: { selected_obfuscation: "off" } should be replaced with
    /// obfuscation_settings: { selected_obfuscation: "auto" }
    #[test]
    fn migrate_seleted_obfuscation_from_off() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"{ "obfuscation_settings": { "selected_obfuscation": "off" } }"#,
        )
        .unwrap();
        migrate_selected_obfuscaton(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"{ "obfuscation_settings": { "selected_obfuscation": "auto" } }"#,
        )
        .unwrap();

        assert_eq!(migrated_settings, expected_settings);
    }
}
