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
    use super::{migrate, migrate_selected_obfuscaton, version_matches};

    pub const V8_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": false,
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
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  },
  "obfuscation_settings": {
    "selected_obfuscation": "off",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "custom_lists": {
    "custom_lists": []
  },
  "api_access_methods": {
    "direct": {
      "id": "5b11a427-a06e-4a06-9864-0d3df7402ee4",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "bf03faf6-229e-4b1e-a7bd-32e0786ca5cb",
      "name": "Mullvad Bridges",
      "enabled": true,
      "access_method": {
        "built_in": "bridge"
      }
    },
    "custom": []
  },
  "allow_lan": false,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "quantum_resistant": "auto",
      "rotation_interval": null
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false,
        "block_malware": false,
        "block_adult_content": false,
        "block_gambling": false,
        "block_social_media": false
      },
      "custom_options": {
        "addresses": []
      }
    }
  },
  "relay_overrides": [],
  "show_beta_releases": true,
  "settings_version": 8
}
"#;

    pub const V9_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": false,
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
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  },
  "obfuscation_settings": {
    "selected_obfuscation": "auto",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "custom_lists": {
    "custom_lists": []
  },
  "api_access_methods": {
    "direct": {
      "id": "5b11a427-a06e-4a06-9864-0d3df7402ee4",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "bf03faf6-229e-4b1e-a7bd-32e0786ca5cb",
      "name": "Mullvad Bridges",
      "enabled": true,
      "access_method": {
        "built_in": "bridge"
      }
    },
    "custom": []
  },
  "allow_lan": false,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "quantum_resistant": "auto",
      "rotation_interval": null
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false,
        "block_malware": false,
        "block_adult_content": false,
        "block_gambling": false,
        "block_social_media": false
      },
      "custom_options": {
        "addresses": []
      }
    }
  },
  "relay_overrides": [],
  "show_beta_releases": true,
  "settings_version": 9
}
"#;

    #[test]
    fn test_v8_to_v9_migration() {
        let mut old_settings = serde_json::from_str(V8_SETTINGS).unwrap();

        assert!(version_matches(&old_settings));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V9_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
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
