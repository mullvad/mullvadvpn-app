use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuantumResistantState {
    Auto,
    On,
    Off,
}

// ======================================================

/// This is an open ended migration. There is no v7 yet!
/// The migrations performed by this function are still backwards compatible.
/// The JSON coming out of this migration can be read by any v6 compatible daemon.
///
/// When further migrations are needed, add them here and if they are not backwards
/// compatible then create v7 and "close" this migration for further modification.
///
/// The `use_pq_safe_psk` tunnel option is replaced by `quantum_resistant`, which
/// is optional. `false` is mapped to `None`. `true` is mapped to `Some(true)`.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    migrate_pq_setting(settings)?;

    // TODO
    // log::info!("Migrating settings format to V7");

    // Note: Not incrementing the version number yet, since this migration is still open
    // for future modification.
    // settings["settings_version"] = serde_json::json!(SettingsVersion::V7);

    Ok(())
}

fn migrate_pq_setting(settings: &mut serde_json::Value) -> Result<()> {
    if let Some(tunnel_options) = settings
        .get_mut("tunnel_options")
        .and_then(|opt| opt.get_mut("wireguard"))
    {
        if let Some(psk_setting) = tunnel_options
            .as_object_mut()
            .ok_or(Error::InvalidSettingsContent)?
            .remove("use_pq_safe_psk")
        {
            if let Some(true) = psk_setting.as_bool() {
                tunnel_options["quantum_resistant"] = serde_json::json!(QuantumResistantState::On);
            } else {
                tunnel_options["quantum_resistant"] =
                    serde_json::json!(QuantumResistantState::Auto);
            }
        }
    }
    Ok(())
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V6 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::{migrate, migrate_pq_setting, version_matches};
    use serde_json;

    pub const V6_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "country": "se"
        }
      },
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": true,
        "entry_location": "any"
      },
      "openvpn_constraints": {
        "port": {
          "only": {
            "protocol": "udp",
            "port": {
              "only": 1195
            }
          }
        }
      }
    }
  },
  "bridge_settings": {
    "normal": {
      "location": "any"
    }
  },
  "obfuscation_settings": {
    "selected_obfuscation": "udp2_tcp",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "allow_lan": true,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "rotation_interval": {
          "secs": 86400,
          "nanos": 0
      },
      "use_pq_safe_psk": false
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false
      },
      "custom_options": {
        "addresses": [
          "1.1.1.1",
          "1.2.3.4"
        ]
      }
    }
  },
  "settings_version": 6
}
"#;

    pub const V7_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "country": "se"
        }
      },
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": true,
        "entry_location": "any"
      },
      "openvpn_constraints": {
        "port": {
          "only": {
            "protocol": "udp",
            "port": {
              "only": 1195
            }
          }
        }
      }
    }
  },
  "bridge_settings": {
    "normal": {
      "location": "any"
    }
  },
  "obfuscation_settings": {
    "selected_obfuscation": "udp2_tcp",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "allow_lan": true,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "rotation_interval": {
          "secs": 86400,
          "nanos": 0
      },
      "quantum_resistant": "auto"
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false
      },
      "custom_options": {
        "addresses": [
          "1.1.1.1",
          "1.2.3.4"
        ]
      }
    }
  },
  "settings_version": 6
}
"#;

    #[test]
    fn test_v6_to_v7_migration() {
        let mut old_settings = serde_json::from_str(V6_SETTINGS).unwrap();

        assert!(version_matches(&mut old_settings));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V7_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }

    /// use_pq_safe_psk=false should be replaced with quantum_resistant=null
    #[test]
    fn test_from_pq_safe_psk_false() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "tunnel_options": {
                "wireguard": {
                    "use_pq_safe_psk": false
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_pq_setting(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "tunnel_options": {
                "wireguard": {
                    "quantum_resistant": "auto"
                }
            }
        }
        "#,
        )
        .unwrap();

        assert_eq!(migrated_settings, expected_settings);
    }

    /// use_pq_safe_psk=true should be replaced with quantum_resistant=true
    #[test]
    fn test_from_pq_safe_psk_true() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "tunnel_options": {
                "wireguard": {
                    "use_pq_safe_psk": true
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_pq_setting(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "tunnel_options": {
                "wireguard": {
                    "quantum_resistant": "on"
                }
            }
        }
        "#,
        )
        .unwrap();

        assert_eq!(migrated_settings, expected_settings);
    }
}
