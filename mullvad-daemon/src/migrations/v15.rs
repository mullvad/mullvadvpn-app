use super::Result;
use mullvad_types::settings::SettingsVersion;
use serde_json::json;

const WIREGUARD_ENTRY_LOCATION_KEY: &str = "entry_location";

/// NOTE: This migration has been closed.
///
/// This migration handles:
/// - Migrates users that have their entry location set to Country("") to any
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V16");

    migrate_entry_location(settings);

    //settings["settings_version"] = serde_json::json!(SettingsVersion::V16);

    Ok(())
}

fn migrate_entry_location(settings: &mut serde_json::Value) -> Option<()> {
    let wireguard_constraints = settings
        .get_mut("relay_settings")
        .and_then(|relay_settings| relay_settings.get_mut("normal"))
        .and_then(|normal_relay_settings| normal_relay_settings.get_mut("wireguard_constraints"))
        .and_then(|wireguard_constraints| wireguard_constraints.as_object_mut())?;

    let entry_location_value = &wireguard_constraints[WIREGUARD_ENTRY_LOCATION_KEY];

    let bad_entry_location = json!({ "only": {
        "location": {
          "country": ""
        }
      }
    });

    if entry_location_value == &bad_entry_location {
        wireguard_constraints.insert(WIREGUARD_ENTRY_LOCATION_KEY.to_string(), json!("any"));
    }

    Some(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V15 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_v15_to_v16_migration_bad_entry_location_replaced() {
        let mut old_settings = json!({
            "relay_settings": {
              "normal": {
                "wireguard_constraints": {
                    "entry_location": {
                        "only": {
                            "location": {
                                "country": ""
                            }
                        }
                    }
                }
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_entry_location(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    #[test]
    fn test_v15_to_v16_migration_any_entry_location_not_replaced() {
        let mut old_settings = json!({
            "relay_settings": {
              "normal": {
                "wireguard_constraints": {
                    "entry_location": "any"
                }
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_entry_location(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    #[test]
    fn test_v15_to_v16_migration_good_entry_location_not_replaced() {
        let mut old_settings = json!({
            "relay_settings": {
              "normal": {
                "wireguard_constraints": {
                    "entry_location": {
                        "only": {
                            "location": {
                                "country": "se"
                            }
                        }
                    }
                }
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_entry_location(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
