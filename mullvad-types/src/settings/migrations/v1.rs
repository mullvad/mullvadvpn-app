use super::Result;
use crate::relay_constraints::Constraint;
use talpid_types::net::TunnelType;


pub(super) struct Migration;

impl super::SettingsMigration for Migration {
    fn version_matches(&self, settings: &mut serde_json::Value) -> bool {
        settings.get("settings_version").is_none()
    }

    fn migrate(&self, settings: &mut serde_json::Value) -> Result<()> {
        log::info!("Migrating settings format to V2");

        let openvpn_constraints = || -> Option<serde_json::Value> {
            settings
                .get("relay_settings")?
                .get("normal")?
                .get("tunnel")?
                .get("only")?
                .get("openvpn")
                .cloned()
        }();
        let wireguard_constraints = || -> Option<serde_json::Value> {
            settings
                .get("relay_settings")?
                .get("normal")?
                .get("tunnel")?
                .get("only")?
                .get("wireguard")
                .cloned()
        }();

        if let Some(relay_settings) = settings.get_mut("relay_settings") {
            if let Some(normal_settings) = relay_settings.get_mut("normal") {
                if let Some(openvpn_constraints) = openvpn_constraints {
                    normal_settings["openvpn_constraints"] = openvpn_constraints;
                    normal_settings["tunnel_protocol"] =
                        serde_json::json!(Constraint::<TunnelType>::Any);
                } else if let Some(wireguard_constraints) = wireguard_constraints {
                    normal_settings["wireguard_constraints"] = wireguard_constraints;
                    normal_settings["tunnel_protocol"] =
                        serde_json::json!(Constraint::Only(TunnelType::Wireguard));
                } else {
                    normal_settings["tunnel_protocol"] =
                        serde_json::json!(Constraint::<TunnelType>::Any);
                }
                if let Some(object) = normal_settings.as_object_mut() {
                    object.remove("tunnel");
                }
            }
        }

        settings["show_beta_releases"] = serde_json::json!(false);
        settings["settings_version"] = serde_json::json!(super::SettingsVersion::V2);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::try_migrate_settings;
    use serde_json;

    pub const NEW_SETTINGS: &str = r#"
{
  "account_token": "1234",
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "country": "se"
        }
      },
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any"
      },
      "openvpn_constraints": {
        "port": {
          "only": {
            "protocol": "udp",
            "port": {
              "only": 53
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
  "bridge_state": "auto",
  "allow_lan": true,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null
    },
    "generic": {
      "enable_ipv6": false
    }
  },
  "settings_version": 5
}
"#;

    const V1_SETTINGS: &str = r#"
{
  "account_token": "1234",
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "country": "se"
        }
      },
      "tunnel": {
        "only": {
          "openvpn": {
            "port": {
              "only": 53
            },
            "protocol": {
              "only": "udp"
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
  "bridge_state": "auto",
  "allow_lan": true,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null
    },
    "generic": {
      "enable_ipv6": false
    }
  }
}
"#;

    const V1_SETTINGS_2019V3: &str = r#"
{
  "account_token": "1234",
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "country": "se"
        }
      },
      "tunnel": {
        "only": {
          "openvpn": {
            "port": {
              "only": 53
            },
            "protocol": {
              "only": "udp"
            }
          }
        }
      }
    }
  },
  "allow_lan": true,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null,
      "proxy": null
    },
    "wireguard": {
      "mtu": null
    },
    "generic": {
      "enable_ipv6": false
    }
  }
}

"#;

    #[test]
    fn test_v1_migration() {
        let migrated_settings =
            try_migrate_settings(V1_SETTINGS.as_bytes()).expect("Migration failed");
        let new_settings = serde_json::from_str(NEW_SETTINGS).unwrap();

        assert_eq!(&migrated_settings, &new_settings);
    }

    #[test]
    fn test_v1_2019v3_migration() {
        let migrated_settings =
            try_migrate_settings(V1_SETTINGS_2019V3.as_bytes()).expect("Migration failed");
        let new_settings = serde_json::from_str(NEW_SETTINGS).unwrap();

        assert_eq!(&migrated_settings, &new_settings);
    }
}
