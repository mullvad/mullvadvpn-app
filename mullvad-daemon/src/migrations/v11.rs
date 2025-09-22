use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;

/// The migration handles:
/// - Renaming of block_when_disconnected option to lockdown_mode.
/// - API access method names must now be unique and duplicates will be renamed.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !(version(settings) == Some(SettingsVersion::V11)) {
        return Ok(());
    }

    log::info!("Migrating settings format to v12");

    migrate_block_when_disconnected(settings)?;
    migrate_duplicated_api_access_method_names(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V12);

    Ok(())
}

fn version(settings: &serde_json::Value) -> Option<SettingsVersion> {
    settings
        .get("settings_version")
        .and_then(|version| serde_json::from_value(version.clone()).ok())
}

fn migrate_block_when_disconnected(settings: &mut serde_json::Value) -> Result<()> {
    let key_name_before = "block_when_disconnected";
    let key_name_after = "lockdown_mode";

    let settings_map = settings
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)?;

    // Get the old key's value and insert the new key with that value
    let value = settings_map
        .get(key_name_before)
        .ok_or(Error::InvalidSettingsContent)?;
    settings_map.insert(key_name_after.to_string(), value.clone());

    // Remove the old key
    settings_map.remove(key_name_before);

    Ok(())
}

fn migrate_duplicated_api_access_method_names(settings: &mut serde_json::Value) -> Result<()> {
    let settings_map = settings
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)?;

    if let Some(api_access_methods) = settings_map
        .get_mut("api_access_methods")
        .and_then(serde_json::Value::as_object_mut)
    {
        if let Some(custom_api_access_methods) = api_access_methods
            .get_mut("custom")
            .and_then(|custom_api_access_methods| custom_api_access_methods.as_array_mut())
        {
            // Vector of all access method's names in use, used to figure out how to rename duplicates
            let mut used_access_method_names: Vec<serde_json::Value> = custom_api_access_methods
                .iter()
                .map(|a| a.get("name").unwrap())
                .cloned()
                .collect();
            for (index, access_method) in custom_api_access_methods.iter_mut().enumerate() {
                if let Some(access_method_name) = access_method.get("name") {
                    // Look up how many access methods previously encountered in the loop that has this name
                    let duplicate_name_count = used_access_method_names
                        .iter()
                        .filter(|existing_access_method_name| {
                            *existing_access_method_name == access_method_name
                        })
                        .count();

                    // If duplicates exist, update the name with a suffix based on the duplicate count and index
                    if duplicate_name_count > 0 {
                        // Decode name from JSON
                        let access_method_name_string: String =
                            serde_json::from_value(access_method_name.clone()).unwrap();
                        // Create a new name by append a suffix to the old name
                        let access_method_name_with_suffix =
                            format!("{access_method_name_string}_{duplicate_name_count}{index}",);
                        // Encode new name as JSON
                        let access_method_name_with_suffix_json =
                            serde_json::to_value(access_method_name_with_suffix).unwrap();

                        // Update the access method's name in settings by removing the string
                        let access_method_map = access_method.as_object_mut().unwrap();
                        access_method_map.remove("name");
                        access_method_map.insert(
                            "name".to_string(),
                            access_method_name_with_suffix_json.clone(),
                        );

                        // Push the renamed access method's name to the vector so we can handle
                        // if a later access method has the name we just used
                        used_access_method_names.push(access_method_name_with_suffix_json);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::migrations::v11::migrate_block_when_disconnected;
    use crate::migrations::v11::migrate_duplicated_api_access_method_names;

    /// "block_when_disconnected" is renamed to "lockdown_mode"
    #[test]
    fn test_v11_to_v12_migration_block_when_disconnected_disabled() {
        let mut old_settings = json!({
            "block_when_disconnected": false,
        });
        migrate_block_when_disconnected(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = json!({
            "lockdown_mode": false,
        });
        assert_eq!(&old_settings, &new_settings);
    }

    #[test]
    fn test_v11_to_v12_migration_block_when_disconnected_enabled() {
        let mut old_settings = json!({
            "block_when_disconnected": true,
        });
        migrate_block_when_disconnected(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = json!({
            "lockdown_mode": true,
        });
        assert_eq!(&old_settings, &new_settings);
    }

    // custom access method's names are renamed if they are not unique
    #[test]
    fn test_v11_to_v12_migration_access_method_name_duplicates() {
        let mut old_settings = json!({
            "api_access_methods": {
              "custom": [
                  {
                  "id": "c2871443-abbc-4fc5-a6b3-7419534a8716",
                  "name": "test_3",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "c2871443-abbc-4fc5-a6b3-7419534a8716",
                  "name": "test_2",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "c2871443-abbc-4fc5-a6b3-7419534a8716",
                  "name": "test",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                 {
                  "id": "c2871443-abbc-4fc5-a6b3-7419534a8717",
                  "name": "test_1",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "3213cde4-dba6-4009-a744-144ae6ecf0bb",
                  "name": "test_2",
                  "enabled": false,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "3213cde4-dba6-4009-a744-144ae6ecf0bb",
                  "name": "test_1",
                  "enabled": false,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "3213cde4-dba6-4009-a744-144ae6ecf0bb",
                  "name": "test",
                  "enabled": false,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "ffdf9900-e843-4298-9478-a9dfbaa63b17",
                  "name": "test",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                }
              ]
            }
        });
        migrate_duplicated_api_access_method_names(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
