use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;
use talpid_types::net::TunnelType;

/// Automatic tunnel protocol has been removed. If the tunnel protocol is set to `any`, it will be
/// migrated to `wireguard`, unless the location is an openvpn relay, in which case it will be
/// migrated to `openvpn`.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !(version(settings) == Some(SettingsVersion::V10)) {
        return Ok(());
    }

    log::info!("Migrating settings format to v11");

    migrate_tunnel_type(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V11);

    Ok(())
}

fn version(settings: &serde_json::Value) -> Option<SettingsVersion> {
    settings
        .get("settings_version")
        .and_then(|version| serde_json::from_value(version.clone()).ok())
}

fn relay_settings(settings: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    settings.get_mut("relay_settings")?.get_mut("normal")
}

fn migrate_tunnel_type(settings: &mut serde_json::Value) -> Result<()> {
    let Some(ref mut normal) = relay_settings(settings) else {
        return Ok(());
    };
    match normal.get_mut("tunnel_protocol") {
        // Already migrated
        Some(serde_json::Value::String(s)) if s == "any" => {
            // If openvpn is selected, migrate to openvpn tunnel type
            // Otherwise, select wireguard
            let hostname = normal
                .get_mut("location")
                .and_then(|location| location.get_mut("only"))
                .and_then(|only| only.get_mut("location"))
                .and_then(|only| only.get_mut("hostname").cloned());

            let protocol = if let Some(serde_json::Value::String(s)) = hostname {
                if s.split('-').any(|token| token == "ovpn") {
                    TunnelType::OpenVpn
                } else {
                    TunnelType::Wireguard
                }
            } else {
                TunnelType::Wireguard
            };

            normal["tunnel_protocol"] = serde_json::json!(protocol);
        }
        // Migrate
        Some(serde_json::Value::Object(ref mut constraint)) => {
            if let Some(tunnel_type) = constraint.get("only") {
                let tunnel_type: TunnelType = serde_json::from_value(tunnel_type.clone())
                    .map_err(|_| Error::InvalidSettingsContent)?;
                normal["tunnel_protocol"] = serde_json::json!(tunnel_type);
            } else {
                return Err(Error::InvalidSettingsContent);
            }
        }
        Some(_) => {
            return Err(Error::InvalidSettingsContent);
        }
        // Unexpected result. Do nothing.
        None => (),
    }
    Ok(())
}
#[cfg(test)]
mod test {
    use mullvad_types::settings::SettingsVersion;

    use super::{migrate, version};

    /// Tunnel protocol "any" is migrated to wireguard
    #[test]
    fn test_v9_to_v10_migration() {
        // TODO: Also test the case where the location is not an openvpn relay and the tunnel type
        // is any
        let mut old_settings = serde_json::from_str(V10_SETTINGS).unwrap();

        assert_eq!(version(&old_settings), Some(SettingsVersion::V10));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V11_SETTINGS).unwrap();
        assert_eq!(version(&new_settings), Some(SettingsVersion::V11));

        eprintln!(
            "old_settings: {}",
            serde_json::to_string_pretty(&old_settings).unwrap()
        );
        eprintln!(
            "new_settings: {}",
            serde_json::to_string_pretty(&new_settings).unwrap()
        );
        assert_eq!(&old_settings, &new_settings);
    }

    /// Tunnel protocol "any" is migrated to openvpn, since the location is an openvpn relay
    #[test]
    fn test_v9_to_v10_migration_openvpn_location() {
        // TODO: Also test the case where the location is not an openvpn relay and the tunnel type
        // is any
        let mut old_settings = serde_json::from_str(V10_SETTINGS_OPENVPN_LOCATION).unwrap();

        assert_eq!(version(&old_settings), Some(SettingsVersion::V10));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value =
            serde_json::from_str(V11_SETTINGS_OPENVPN_LOCATION).unwrap();
        assert_eq!(version(&new_settings), Some(SettingsVersion::V11));

        eprintln!(
            "old_settings: {}",
            serde_json::to_string_pretty(&old_settings).unwrap()
        );
        eprintln!(
            "new_settings: {}",
            serde_json::to_string_pretty(&new_settings).unwrap()
        );
        assert_eq!(&old_settings, &new_settings);
    }

    /// Tunnel protocol is set to any, but the location is not an openvpn relay
    pub const V10_SETTINGS: &str = r#"
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
              "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
              "name": "Direct",
              "enabled": true,
              "access_method": {
                "built_in": "direct"
              }
            },
            "mullvad_bridges": {
              "id": "92135711-534d-4950-963d-93e446a792e4",
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
          "settings_version": 10
        }
        "#;

    /// Tunnel protocol is migrated to wireguard
    pub const V11_SETTINGS: &str = r#"
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
              "tunnel_protocol": "wireguard",
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
              "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
              "name": "Direct",
              "enabled": true,
              "access_method": {
                "built_in": "direct"
              }
            },
            "mullvad_bridges": {
              "id": "92135711-534d-4950-963d-93e446a792e4",
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
          "settings_version": 11
        }
        "#;

    /// This settings blob contains no constraint for tunnel type
    pub const V10_SETTINGS_OPENVPN_LOCATION: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": "at-vie-ovpn-001"
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
      "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "92135711-534d-4950-963d-93e446a792e4",
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
  "settings_version": 10
}
"#;

    /// This settings blob does not contain an "any" tunnel type
    pub const V11_SETTINGS_OPENVPN_LOCATION: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": "at-vie-ovpn-001"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "openvpn",
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
      "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "92135711-534d-4950-963d-93e446a792e4",
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
  "settings_version": 11
}
"#;
}
