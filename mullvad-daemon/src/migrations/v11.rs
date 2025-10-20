use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;

/// The migration handles:
/// - Renaming of block_when_disconnected option to lockdown_mode.
/// - API access method names must now be unique and duplicates will be renamed.
/// - Removing the Automatic option from the quantum resistance setting. The default is now "On".
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !(version(settings) == Some(SettingsVersion::V11)) {
        return Ok(());
    }

    log::info!("Migrating settings format to v12");

    migrate_block_when_disconnected(settings)?;
    migrate_duplicated_api_access_method_names(settings)?;
    migrate_quantum_resistance(settings)?;

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

fn generate_access_method_name_initial_suffix(
    access_method_names: &[impl AsRef<str>],
    access_method_name: &String,
) -> usize {
    let access_method_name_count = access_method_names
        .iter()
        .filter(|name| name.as_ref() == access_method_name)
        .count();

    let mut suffix = 1;
    if access_method_name_count > 1 {
        suffix = access_method_name_count - 1
    }

    suffix
}

/// Only consider renaming access methods with a duplicate name if it has a higher index
/// thab other access methods. This is to ensure that older entries' names are preserved,
/// in favor of renaming newer access methods.
fn get_should_rename_api_access_method(
    access_method_names: &[impl AsRef<str>],
    access_method_name: &String,
    access_method_name_index: usize,
) -> bool {
    access_method_names.iter().enumerate().any(|(index, name)| {
        access_method_name_index > index && name.as_ref() == *access_method_name
    })
}

fn generate_access_method_name(
    access_method_names: &[impl AsRef<str>],
    access_method_name: &String,
    access_method_name_index: usize,
    access_method_name_suffix: usize,
) -> String {
    // Generate a new name for the access method
    let generated_access_method_name = format!("{access_method_name}_{access_method_name_suffix}");

    // Verify if the generated name is unique or if a new name should be generated
    let should_rename_api_access_method = get_should_rename_api_access_method(
        access_method_names,
        &generated_access_method_name,
        access_method_name_index,
    );
    if should_rename_api_access_method {
        // Increment the suffix for the next attempt to generate a new access method name
        generate_access_method_name(
            access_method_names,
            access_method_name,
            access_method_name_index,
            access_method_name_suffix + 1,
        )
    } else {
        generated_access_method_name
    }
}

fn migrate_duplicated_api_access_method_names(settings: &mut serde_json::Value) -> Result<()> {
    let settings_map = settings
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)?;

    let mut custom_api_access_methods: Vec<&mut String> = settings_map
        .get_mut("api_access_methods")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|api_access_method| api_access_method.get_mut("custom")?.as_array_mut())
        .into_iter()
        .flat_map(|array| array.iter_mut())
        // Take a &mut to each custom api access method name as a String
        .filter_map(|custom_api_access_method| custom_api_access_method.as_object_mut()?.get_mut("name"))
        .filter_map(|custom_api_access_method| match custom_api_access_method {
            serde_json::Value::String(custom_api_access_method_name) => {
                Some(custom_api_access_method_name)
            }
            _ => None,
        })
        .collect();

    for index in 0..custom_api_access_methods.len() {
        let access_method_name = &*custom_api_access_methods[index];

        let should_rename_api_access_method = get_should_rename_api_access_method(
            &custom_api_access_methods,
            access_method_name,
            index,
        );
        if should_rename_api_access_method {
            let access_method_name_suffix = generate_access_method_name_initial_suffix(
                &custom_api_access_methods,
                access_method_name,
            );

            let generated_access_method_name = generate_access_method_name(
                &custom_api_access_methods,
                access_method_name,
                index,
                access_method_name_suffix,
            );

            // Update the access method's name to the new unique name that was generated
            *custom_api_access_methods[index] = generated_access_method_name;
        }
    }

    Ok(())
}
/// Map "quantum_resistant": "auto" -> "quantum_resistant": "on".
fn migrate_quantum_resistance(settings: &mut serde_json::Value) -> Result<()> {
    use serde_json::Value;
    // settings.tunnel_options.wireguard
    fn wg(settings: &mut Value) -> Option<&mut Value> {
        settings
            .as_object_mut()?
            .get_mut("tunnel_options")?
            .get_mut("wireguard")
    }
    let wg = wg(settings).ok_or(Error::InvalidSettingsContent)?;
    match wg.get_mut("quantum_resistant") {
        Some(quantum_resistance) => {
            if quantum_resistance == "auto" {
                *quantum_resistance = "on".into();
            }
        }
        None => {
            // Believe it or not, the PQ setting is not guaranteed to exist coming from an earlier
            // settings version, because it was never added through a settings migration!
            // I'll go ahead and fix that right here, but going forward we should be more cautious
            // about *not* adding certain settings via migrations. Not doing so means that we rely on
            // the implemenation of Settings::default to fill in all the missing details, which might
            // be ok..
            wg["quantum_resistant"] = "on".into();
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

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

    /// quantum resistant setting is migrated from auto to on.
    #[test]
    fn test_v11_to_v12_migration_pq_auto_to_on() {
        let mut old_settings = json!({
            "tunnel_options": {
              "wireguard": {
                "quantum_resistant": "auto"
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_quantum_resistance(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    /// quantum resistant setting is set to on if it does not exist.
    #[test]
    fn test_v11_to_v12_migration_pq_default_to_on() {
        let mut old_settings = json!({
            "tunnel_options": {
              "wireguard": { }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_quantum_resistance(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
