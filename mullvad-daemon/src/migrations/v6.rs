use super::{Error, Result};
use mullvad_types::{relay_constraints::Constraint, settings::SettingsVersion};
use serde::{Deserialize, Serialize};

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

/// This is a closed migration.
///
/// The `use_pq_safe_psk` tunnel option is replaced by `quantum_resistant`, which
/// is optional. `false` is mapped to `None`. `true` is mapped to `Some(true)`.
///
/// Migrate WireGuard over TCP port setting away from Only(443) (to auto),
/// since it's no longer a valid port.
///
/// Migrate location constraints from `GeographicLocationConstraint` to `LocationConstraint`.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    migrate_pq_setting(settings)?;

    migrate_udp2tcp_port_443(settings);

    migrate_location_constraint(settings)?;

    log::info!("Migrating settings format to V7");

    settings["settings_version"] = serde_json::json!(SettingsVersion::V7);

    Ok(())
}

fn migrate_location_constraint(settings: &mut serde_json::Value) -> Result<()> {
    if let Some(location) = settings
        .get_mut("relay_settings")
        .and_then(|relay_settings| relay_settings.get_mut("normal"))
        .and_then(|normal_relay_settings| normal_relay_settings.get_mut("location"))
    {
        wrap_location(location)?;
    }

    if let Some(location) = settings
        .get_mut("relay_settings")
        .and_then(|relay_settings| relay_settings.get_mut("normal"))
        .and_then(|normal_relay_settings| normal_relay_settings.get_mut("wireguard_constraints"))
        .and_then(|normal_relay_settings| normal_relay_settings.get_mut("entry_location"))
    {
        wrap_location(location)?;
    }

    if let Some(location) = settings
        .get_mut("bridge_settings")
        .and_then(|relay_settings| relay_settings.get_mut("normal"))
        .and_then(|normal_relay_settings| normal_relay_settings.get_mut("location"))
    {
        wrap_location(location)?;
    }

    Ok(())
}

fn wrap_location(location: &mut serde_json::Value) -> Result<()> {
    if let Some(only) = location.get_mut("only") {
        only["location"] = only.clone();
        let only = only.as_object_mut().ok_or(Error::InvalidSettingsContent)?;
        only.remove("country");
        only.remove("city");
        only.remove("hostname");
    }
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

/// If udp2tcp port constraint is set to `Only(443)`, change that to `Any`
fn migrate_udp2tcp_port_443(settings: &mut serde_json::Value) -> Option<()> {
    let port_constraint = settings
        .get_mut("obfuscation_settings")?
        .get_mut("udp2tcp")?
        .get_mut("port")?;
    if port_constraint == &serde_json::json!(Constraint::Only(443)) {
        log::info!("Migrating udp2tcp port setting from 443 -> any");
        *port_constraint = serde_json::json!(Constraint::<u16>::Any);
    }
    None
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V6 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::{migrate, migrate_location_constraint, migrate_pq_setting, version_matches};

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
      "port": {
        "only": 443
      }
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
          "location": {
            "country": "se"
          }
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
  "settings_version": 7
}
"#;

    #[test]
    fn test_v6_to_v7_migration() {
        let mut old_settings = serde_json::from_str(V6_SETTINGS).unwrap();

        assert!(version_matches(&old_settings));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V7_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }

    /// For relay settings
    /// location: { only: { country : "se" } } should be replaced with
    /// location: { only: { location: { country: "se" } } }
    #[test]
    fn test_from_relay_settings_location_constraint_country() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "country": "se"
                        }
                    }
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_location_constraint(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "location": {
                                "country": "se"
                            }
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

    /// For relay settings
    /// location: { only: { country : "se", city: "got" } } should be replaced with
    /// location: { only: { location: { country: "se", city: "got" } } }
    #[test]
    fn test_from_relay_settings_location_constraint_city() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "country": "se",
                            "city": "got"
                        }
                    }
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_location_constraint(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "location": {
                                "country": "se",
                                "city": "got"
                            }
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

    /// For relay settings
    /// location: { only: { country : "se", city: "got", hostname: "se-got-wg-001" } } should be
    /// replaced with location: { only: { location: { country: "se", city: "got", hostname:
    /// "se-got-wg-001" } } }
    #[test]
    fn test_from_relay_settings_location_constraint_hostname() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "country": "se",
                            "city": "got",
                            "hostname": "se-got-wg-001"
                        }
                    }
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_location_constraint(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "location": {
                                "country": "se",
                                "city": "got",
                                "hostname": "se-got-wg-001"
                            }
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

    /// For bridge settings
    /// location: { only: { country : "se", city: "got", hostname: "se-got-wg-001" } } should be
    /// replaced with location: { only: { location: { country: "se", city: "got", hostname:
    /// "se-got-wg-001" } } }
    #[test]
    fn test_from_bridge_location_constraint_hostname() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "bridge_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "country": "se",
                            "city": "got",
                            "hostname": "se-got-wg-001"
                        }
                    }
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_location_constraint(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "bridge_settings": {
                "normal": {
                    "location": {
                        "only": {
                            "location": {
                                "country": "se",
                                "city": "got",
                                "hostname": "se-got-wg-001"
                            }
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

    /// For wireguard constraints
    /// location: { only: { country : "se", city: "got", hostname: "se-got-wg-001" } } should be
    /// replaced with location: { only: { location: { country: "se", city: "got", hostname:
    /// "se-got-wg-001" } } }
    #[test]
    fn test_from_wireguard_constraint_location_constraint_hostname() {
        let mut migrated_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "wireguard_constraints": {
                        "entry_location": {
                            "only": {
                                "country": "se",
                                "city": "got",
                                "hostname": "se-got-wg-001"
                            }
                        }
                    }
                }
            }
        }
        "#,
        )
        .unwrap();
        migrate_location_constraint(&mut migrated_settings).unwrap();

        let expected_settings: serde_json::Value = serde_json::from_str(
            r#"
        {
            "relay_settings": {
                "normal": {
                    "wireguard_constraints": {
                        "entry_location": {
                            "only": {
                                "location": {
                                    "country": "se",
                                    "city": "got",
                                    "hostname": "se-got-wg-001"
                                }
                            }
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
