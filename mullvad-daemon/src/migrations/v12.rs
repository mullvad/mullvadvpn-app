use super::Result;
use mullvad_types::settings::SettingsVersion;

/// This version introduces 2 new fields to the [mullvad_constraints::WireguardConstraints] struct:
/// pub entry_providers: Constraint<Providers>,
/// pub entry_ownership: Constraint<Ownership>,
/// When set, these filters apply to the entry relay when multihop is used.
/// A migration is needed to transfer the current providers and ownership to these new fields
/// so that the user's current filters don't change.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V13");

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

#[cfg(test)]
mod test {
    use super::{migrate, version_matches};

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
  "settings_version": 12
}
"#;

    #[test]
    fn test_v12_to_v13_migration() {
        let mut old_settings = serde_json::from_str(V12_SETTINGS).unwrap();

        assert!(version_matches(&old_settings));

        migrate(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
