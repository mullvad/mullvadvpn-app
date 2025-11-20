use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;

/// NOTE: This migration has been closed.
///
/// This migration handles:
/// - Removing the Automatic option from the quantum resistance setting. The default is now "On".
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V14");

    migrate_quantum_resistance(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V14);

    Ok(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V13 as u64)
        .unwrap_or(false)
}

/// Map "quantum_resistant": "auto" -> "quantum_resistant": "on".
fn migrate_quantum_resistance(settings: &mut serde_json::Value) -> Result<()> {
    // settings.tunnel_options.wireguard
    let wg = settings
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)?
        .get_mut("tunnel_options")
        .ok_or(Error::MissingKey("'tunnel_options' missing"))?
        .get_mut("wireguard")
        .ok_or(Error::MissingKey("'wireguard' missing"))?;
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

    const V13_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "fr"
          }
        }
      },
      "providers": {
        "only": {
          "providers": [
            "Blix",
            "Creanova"
          ]
        }
      },
      "ownership": {
        "only": "MullvadOwned"
      },
      "tunnel_protocol": "wireguard",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "allowed_ips": "any",
        "use_multihop": true,
        "entry_location": {
          "only": {
            "location": {
              "country": "se"
            }
          }
        }
      },
      "openvpn_constraints": {
        "port": "any"
      }
    }
  },
  "tunnel_options": {
    "wireguard": {
      "quantum_resistant": "auto"
    }
  },
  "settings_version": 13
}
"#;

    #[test]
    fn test_v13_to_v14_migration() -> Result<()> {
        let mut old_settings = serde_json::from_str(V13_SETTINGS).unwrap();

        assert!(version_matches(&old_settings));

        migrate(&mut old_settings)?;
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        Ok(())
    }

    /// If a client already have gone through a migration of PQ settings previously, make
    /// sure that running this migration does not break. The mathematical name for this property
    /// would be idempotancy.
    #[test]
    fn test_migration_identity() {
        // Possible input values:
        // * "quantum_resistant": "on"
        // * "quantum_resistant": "off"
        // Possible output values:
        // * "quantum_resistant": "on" -> "on"
        // * "quantum_resistant": "off" -> "off"
        {
            // "on" -> "on"
            let mut on_to_on = json!({
                "tunnel_options": {
                  "wireguard": {
                    "quantum_resistant": "on"
                  }
                },
                "settings_version": 13
            });
            migrate(&mut on_to_on).unwrap();
            insta::assert_snapshot!(serde_json::to_string_pretty(&on_to_on).unwrap());
        }
        {
            // "off" -> "off"
            let mut off_to_off = json!({
                "tunnel_options": {
                  "wireguard": {
                    "quantum_resistant": "off"
                  }
                },
                "settings_version": 13
            });
            migrate(&mut off_to_off).unwrap();
            insta::assert_snapshot!(serde_json::to_string_pretty(&off_to_off).unwrap());
        }
    }

    /// quantum resistant setting is migrated from auto to on.
    #[test]
    fn test_v13_to_v14_migration_pq_auto_to_on() {
        let mut old_settings = json!({
            "tunnel_options": {
              "wireguard": {
                "quantum_resistant": "auto"
              }
            }
        });
        migrate_quantum_resistance(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    /// quantum resistant setting is set to on if it does not exist.
    #[test]
    fn test_v13_to_v14_migration_pq_default_to_on() {
        let mut old_settings = json!({
            "tunnel_options": {
              "wireguard": { }
            }
        });
        migrate_quantum_resistance(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
