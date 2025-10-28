use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;

/// This migration handles:
/// - Removing the Automatic option from the quantum resistance setting. The default is now "On".
/// - Introduces 2 new fields to the [mullvad_constraints::WireguardConstraints] struct:
///   pub entry_providers: Constraint<Providers>,
///   pub entry_ownership: Constraint<Ownership>,
///   When set, these filters apply to the entry relay when multihop is used.
///   A migration is needed to transfer the current providers and ownership to these new fields
///   so that the user's current filters don't change.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V13");

    migrate_quantum_resistance(settings)?;
    migrate_filters_to_new_entry_only_filters(settings);

    settings["settings_version"] = serde_json::json!(SettingsVersion::V13);

    Ok(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V12 as u64)
        .unwrap_or(false)
}

fn migrate_filters_to_new_entry_only_filters(settings: &mut serde_json::Value) -> Option<()> {
    let normal = settings.get_mut("relay_settings")?.get_mut("normal")?;
    let providers = normal.get("providers")?.clone();
    let ownership = normal.get("ownership")?.clone();

    let wireguard_constraints = normal.get_mut("wireguard_constraints")?.as_object_mut()?;

    wireguard_constraints.insert("entry_providers".to_string(), providers);
    wireguard_constraints.insert("entry_ownership".to_string(), ownership);

    Some(())
}

/// Map "quantum_resistant": "auto" -> "quantum_resistant": "on".
fn migrate_quantum_resistance(settings: &mut serde_json::Value) -> Result<()> {
    use serde_json::Value;
    // settings.tunnel_options.wireguard
    fn wg(settings: &mut Value) -> Result<&mut Value> {
        settings
            .as_object_mut()
            .ok_or(Error::InvalidSettingsContent)?
            .get_mut("tunnel_options")
            .ok_or(Error::MissingKey("'tunnel_options' missing"))?
            .get_mut("wireguard")
            .ok_or(Error::MissingKey("'wireguard' missing"))
    }
    let wg = wg(settings)?;
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

    const V12_SETTINGS: &str = r#"
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
  "settings_version": 12
}
"#;

    #[test]
    fn test_v12_to_v13_migration() -> Result<()> {
        let mut old_settings = serde_json::from_str(V12_SETTINGS).unwrap();

        assert!(version_matches(&old_settings));

        migrate(&mut old_settings)?;
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        Ok(())
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
