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

fn decode_json_to_string(json_string: &serde_json::Value) -> String {
    serde_json::from_value(json_string.clone()).unwrap()
}

fn encode_string_to_json(string: String) -> serde_json::Value {
    serde_json::to_value(string.clone()).unwrap()
}

fn get_api_access_method_name(api_access_method: &serde_json::Value) -> String {
    let json_name = api_access_method.get("name").unwrap();
    decode_json_to_string(json_name)
}

fn get_api_access_method_names(api_access_methods: &Vec<serde_json::Value>) -> Vec<String> {
    api_access_methods
        .iter()
        .map(|api_access_method| get_api_access_method_name(api_access_method))
        .collect()
}

/// Only consider renaming access methods with a duplicate name if it has a higher index
/// other access method. This is to ensure that older entries' names are preserved,
/// in favor of renaming newer access methods.
fn get_should_rename_api_access_method(
    access_method_names: &[String],
    access_method_name: &String,
    access_method_name_index: usize,
) -> bool {
    access_method_names
        .iter()
        .enumerate()
        .any(|(index, name)| access_method_name_index > index && *name == *access_method_name)
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
        for index in 0..custom_api_access_methods.len() {
            let custom_api_access_method_names =
                get_api_access_method_names(custom_api_access_methods);
            let custom_api_access_method_name = &custom_api_access_method_names[index];

            // Check if an access method should be renamed
            let mut attempts = 0;
            let mut generated_custom_api_access_method_name = custom_api_access_method_name.clone();
            while get_should_rename_api_access_method(
                &custom_api_access_method_names,
                &generated_custom_api_access_method_name,
                index,
            ) {
                // Generate a new name for the access method
                let suffix = attempts + 1;
                generated_custom_api_access_method_name =
                    format!("{custom_api_access_method_name}_{suffix}");
                attempts += 1;
            }

            // Update the access method's name to the new unique name if one was generated
            if generated_custom_api_access_method_name != *custom_api_access_method_name {
                custom_api_access_methods[index]["name"] =
                    encode_string_to_json(generated_custom_api_access_method_name);
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
                  "id": "d284a9d5-307b-4959-94a6-89fef8187807",
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
