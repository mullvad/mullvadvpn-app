use super::Result;
use mullvad_types::settings::SettingsVersion;
use serde_json::json;

const WIREGUARD_PORT_OLD_KEY: &str = "port";
const WIREGUARD_PORT_NEW_KEY: &str = "wireguard_port";

/// This migration handles:
/// - Migrates the WireGuard port from WireGuard constraints in relay settings
///   to obfuscation settings.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V14");

    migrate_wireguard_port(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V14);

    Ok(())
}

fn migrate_wireguard_port(settings: &mut serde_json::Value) -> Result<()> {
    if let Some(wireguard_constraints) = settings
        .get_mut("relay_settings")
        .and_then(|relay_settings| relay_settings.get_mut("normal"))
        .and_then(|normal_relay_settings| normal_relay_settings.get_mut("wireguard_constraints"))
        .and_then(|wireguard_constraints| wireguard_constraints.as_object_mut())
    {
        if let Some(port) = wireguard_constraints.get(WIREGUARD_PORT_OLD_KEY) {
            let wireguard_port = port.clone();
            wireguard_constraints.remove(WIREGUARD_PORT_OLD_KEY);

            if let Some(obfuscation_settings) = settings
                .get_mut("obfuscation_settings")
                .and_then(|obfuscation_settings| obfuscation_settings.as_object_mut())
            {
                obfuscation_settings.insert(
                    WIREGUARD_PORT_NEW_KEY.to_string(),
                    json!({"port": wireguard_port}),
                );
            }
        }
    }

    Ok(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V13 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::migrations::v13::migrate_wireguard_port;

    #[test]
    fn test_v13_to_v14_migration_wireguard_port_any_selected_obfuscation_custom() {
        let mut old_settings = json!({
            "obfuscation_settings": {
                "selected_obfuscation": "shadowsocks"
            },
            "relay_settings": {
              "normal": {
                "wireguard_constraints": {
                    "port": "any"
                }
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_wireguard_port(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    #[test]
    fn test_v13_to_v14_migration_wireguard_port_any_selected_obfuscation_auto() {
        let mut old_settings = json!({
            "obfuscation_settings": {
                "selected_obfuscation": "auto"
            },
            "relay_settings": {
              "normal": {
                "wireguard_constraints": {
                    "port": "any"
                }
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_wireguard_port(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    #[test]
    fn test_v13_to_v14_migration_wireguard_port_value() {
        let mut old_settings = json!({
            "obfuscation_settings": {},
            "relay_settings": {
              "normal": {
                "wireguard_constraints": {
                    "port": {
                        "only": 53
                    }
                }
              }
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_wireguard_port(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    #[test]
    fn test_v13_to_v14_migration_relay_settings_custom() {
        let mut old_settings = json!({
            "obfuscation_settings": {},
            "relay_settings": {
              "custom": {}
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        migrate_wireguard_port(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
