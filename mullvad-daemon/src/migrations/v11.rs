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

fn get_is_access_method_name_duplicate(
    used_access_method_names: &[String],
    access_method_name: &String,
) -> bool {
    used_access_method_names
        .iter()
        .filter(|used_name| *used_name == access_method_name)
        .count()
        > 0
}

fn generate_unique_access_method_name(
    used_access_method_names: &[String],
    access_method_name: &String,
    suffix: usize,
) -> String {
    // Create a new name by appending the suffix to the old name
    let unique_access_method_name = format!("{access_method_name}_{suffix}",);

    // Check if the new name is unique
    let is_access_method_name_duplicate =
        get_is_access_method_name_duplicate(used_access_method_names, &unique_access_method_name);
    if is_access_method_name_duplicate {
        // If the generated name is also a duplicate, increment the suffix and try again
        generate_unique_access_method_name(used_access_method_names, access_method_name, suffix + 1)
    } else {
        unique_access_method_name
    }
}

fn migrate_duplicated_api_access_method_names(settings: &mut serde_json::Value) -> Result<()> {
    let settings_map = settings
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)?;

    if let Some(custom_api_access_methods) = settings_map
        .get_mut("api_access_methods")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|api_access_method| api_access_method.get_mut("custom"))
        .and_then(|custom_api_access_methods| custom_api_access_methods.as_array_mut())
    {
        // Vector to hold all API access method's names in use, it is used to figure
        // if an access method is unique or if it should be renamed
        let mut used_access_method_names: Vec<String> = Vec::new();
        for access_method in custom_api_access_methods.iter_mut() {
            if let Some(access_method_name) = access_method.get("name") {
                // Decode name from JSON
                let access_method_name_string: String =
                    serde_json::from_value(access_method_name.clone()).unwrap();

                let is_access_method_name_duplicate = get_is_access_method_name_duplicate(
                    &used_access_method_names,
                    &access_method_name_string,
                );
                // Check if access method name is a duplicate,
                // and rename it if
                if is_access_method_name_duplicate {
                    // Generate a new name for the access method
                    let unique_access_method_name = generate_unique_access_method_name(
                        &used_access_method_names,
                        &access_method_name_string,
                        1,
                    );

                    // Encode new name as JSON
                    let unique_access_method_name_json =
                        serde_json::to_value(unique_access_method_name.clone()).unwrap();

                    // Update the access method's name in settings by first removing the name key
                    // and then inserting the new value for the name key
                    let access_method_map = access_method.as_object_mut().unwrap();
                    access_method_map.remove("name");
                    access_method_map.insert("name".to_string(), unique_access_method_name_json);

                    // Add the access method's new name to the vector so we
                    // can handle if another access method would happen to use
                    // the new generated name.
                    used_access_method_names.push(unique_access_method_name);
                } else {
                    // If the access method's current name was unique, we add the
                    // name to the used names vector so we can handle if another
                    // access method also uses that name.
                    used_access_method_names.push(access_method_name_string);
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
                  "id": "90d35296-3823-4805-8926-720fff53c752",
                  "name": "test_3",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:80",
                        "password": "",
                        "cipher": "aes-128-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "d879f6e9-c052-4452-8e53-088183d01c0a",
                  "name": "test_2",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "secret",
                        "cipher": "aes-256-cfb"
                      }
                    }
                  }
                },
                {
                  "id": "7b49cef8-5a9f-4bbc-9bee-00841edc98e9",
                  "name": "test",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:443",
                        "password": "",
                        "cipher": "aes-256-gcm"
                      }
                    }
                  }
                },
                 {
                  "id": "0bafc4ed-cd4f-4368-b067-74527f42451b",
                  "name": "test_1",
                  "enabled": true,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:80",
                        "password": "",
                        "cipher": "aes-128-gcm"
                      }
                    }
                  }
                },
                {
                  "id": "09d032bc-7e3a-4d85-a63f-528b6c4b890e",
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
                  "id": "4471b3f5-a87e-4355-aea8-72c4c4936479",
                  "name": "test_1",
                  "enabled": false,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:80",
                        "password": "secret",
                        "cipher": "aes-256-gcm"
                      }
                    }
                  }
                },
                {
                  "id": "6f8db7c3-2258-46c0-8b7d-2016dd9e5739",
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
                  "id": "6f8db7c3-2258-46c0-8b7d-2016dd9e5739",
                  "name": "other_name",
                  "enabled": false,
                  "access_method": {
                    "custom": {
                      "shadowsocks": {
                        "endpoint": "127.0.0.1:9090",
                        "password": "",
                        "cipher": "aes-256-cfb"
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
                        "endpoint": "127.0.0.1:8080",
                        "password": "",
                        "cipher": "aes-128-gcm"
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
