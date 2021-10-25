use super::Result;
use mullvad_types::{relay_constraints::Constraint, settings::SettingsVersion};
use talpid_types::net::TunnelType;


pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

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
    settings["settings_version"] = serde_json::json!(SettingsVersion::V2);

    Ok(())
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings.get("settings_version").is_none()
}

#[cfg(test)]
mod test {
    use super::{migrate, version_matches};
    use serde_json;

    pub const V2_SETTINGS: &str = r#"
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
      "openvpn_constraints": {
        "port": {
          "only": 53
        },
        "protocol": {
          "only": "udp"
        }
      }
    }
  },
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
  "show_beta_releases": false,
  "settings_version": 2
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

    #[test]
    fn test_v1_migration() {
        let mut old_settings = serde_json::from_str(V1_SETTINGS).unwrap();

        assert!(version_matches(&mut old_settings));

        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V2_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }

    #[test]
    fn test_v1_2019v3_migration() {
        let mut old_settings = serde_json::from_str(V1_SETTINGS_2019V3).unwrap();

        assert!(version_matches(&mut old_settings));

        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V2_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }
}
