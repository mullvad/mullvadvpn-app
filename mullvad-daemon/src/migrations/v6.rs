use super::{Error, Result};
use mullvad_types::{relay_constraints::Constraint, settings::SettingsVersion};

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

// ======================================================

/// This is an open ended migration. There is no v7 yet!
/// The migrations performed by this function are still backwards compatible.
/// The JSON coming out of this migration can be read by any v6 compatible daemon.
///
/// When further migrations are needed, add them here and if they are not backwards
/// compatible then create v7 and "close" this migration for further modification.
///
/// The `use_pq_safe_psk` tunnel option is replaced by `quantum_resistant` relay
/// constraint, which is optional. `false` is mapped to `Constraint::Any`.
/// `true` is mapped to `Constraint::Only(true)`.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    migrate_pq_setting(settings)?;

    // TODO
    //log::info!("Migrating settings format to V7");

    // Note: Not incrementing the version number yet, since this migration is still open
    // for future modification.
    //settings["settings_version"] = serde_json::json!(SettingsVersion::V7);

    Ok(())
}

fn get_wireguard_tunnel_options(
    settings: &mut serde_json::Value,
) -> Option<&mut serde_json::Value> {
    if let Some(tunnel_options) = settings.get_mut("tunnel_options") {
        return tunnel_options.get_mut("wireguard");
    }
    None
}

fn get_wireguard_relay_settings(
    settings: &mut serde_json::Value,
) -> Option<&mut serde_json::Value> {
    if let Some(relay_settings) = settings.get_mut("relay_settings") {
        if let Some(normal) = relay_settings.get_mut("normal") {
            return normal.get_mut("wireguard_constraints");
        }
    }
    None
}

fn migrate_pq_setting(settings: &mut serde_json::Value) -> Result<()> {
    let mut new_pq_constraint = None;

    if let Some(tunnel_options) = get_wireguard_tunnel_options(settings) {
        if let Some(psk_setting) = tunnel_options.get_mut("use_pq_safe_psk") {
            if let Some(true) = psk_setting.as_bool() {
                new_pq_constraint = Some(Constraint::Only(true));
            } else {
                new_pq_constraint = Some(Constraint::Any);
            }
        }
        tunnel_options
            .as_object_mut()
            .ok_or(Error::NoMatchingVersion)?
            .remove("use_pq_safe_psk");
    }

    if let Some(pq_constraint) = new_pq_constraint {
        if let Some(wg_settings) = get_wireguard_relay_settings(settings) {
            wg_settings["quantum_resistant"] = serde_json::json!(pq_constraint);
        } else {
            // This should be because custom tunnels are used. In that case
            // PQ is disabled anyhow, so this is alright.
            log::warn!(
                "Losing quantum resistant setting because relay constraints could not be obtained"
            );
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
        "entry_location": "any",
        "quantum_resistant": "any"
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
      }
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
          },
          "relay_settings": {
            "normal": {
              "wireguard_constraints": {
              }
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
            }
          },
          "relay_settings": {
            "normal": {
              "wireguard_constraints": {
                "quantum_resistant": "any"
              }
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
          },
          "relay_settings": {
            "normal": {
              "wireguard_constraints": {
              }
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
            }
          },
          "relay_settings": {
            "normal": {
              "wireguard_constraints": {
                "quantum_resistant": {
                  "only": true
                }
              }
            }
          }
        }
        "#,
        )
        .unwrap();

        assert_eq!(migrated_settings, expected_settings);
    }
}
