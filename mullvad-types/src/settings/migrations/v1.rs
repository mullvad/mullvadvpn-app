use super::{Error, Result};
use crate::{
    custom_tunnel::CustomTunnelEndpoint,
    relay_constraints::{
        Constraint, LocationConstraint, OpenVpnConstraints, RelaySettings as NewRelaySettings,
        WireguardConstraints,
    },
};
use serde::{Deserialize, Serialize};
use talpid_types::net::TunnelType;


pub(super) struct Migration;

impl super::SettingsMigration for Migration {
    fn version_matches(&self, settings: &mut serde_json::Value) -> bool {
        settings.get("settings_version").is_none()
    }

    fn migrate(&self, settings: &mut serde_json::Value) -> Result<()> {
        log::info!("Migrating settings format to V2");

        let old_relay_settings: RelaySettings =
            serde_json::from_value(settings["relay_settings"].clone())
                .map_err(Error::ParseError)?;
        let new_relay_settings = migrate_relay_settings(old_relay_settings);

        settings["relay_settings"] = serde_json::json!(new_relay_settings);
        settings["show_beta_releases"] = serde_json::json!(false);
        settings["settings_version"] = serde_json::json!(super::SettingsVersion::V2);

        Ok(())
    }
}

fn migrate_relay_settings(relay_settings: RelaySettings) -> NewRelaySettings {
    match relay_settings {
        RelaySettings::CustomTunnelEndpoint(endpoint) => {
            crate::relay_constraints::RelaySettings::CustomTunnelEndpoint(endpoint)
        }
        RelaySettings::Normal(old_constraints) => {
            let mut new_constraints = crate::relay_constraints::RelayConstraints {
                location: old_constraints.location,
                ..Default::default()
            };
            match old_constraints.tunnel {
                Constraint::Any => (),
                Constraint::Only(TunnelConstraints::OpenVpn(constraints)) => {
                    new_constraints.openvpn_constraints = constraints;
                }
                Constraint::Only(TunnelConstraints::Wireguard(constraints)) => {
                    new_constraints.wireguard_constraints = constraints;
                    new_constraints.tunnel_protocol = Constraint::Only(TunnelType::Wireguard);
                }
            };
            crate::relay_constraints::RelaySettings::Normal(new_constraints)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelaySettings {
    CustomTunnelEndpoint(CustomTunnelEndpoint),
    Normal(RelayConstraints),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RelayConstraints {
    pub location: Constraint<LocationConstraint>,
    pub tunnel: Constraint<TunnelConstraints>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum TunnelConstraints {
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnConstraints),
    #[serde(rename = "wireguard")]
    Wireguard(WireguardConstraints),
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
          "only": 53
        },
        "protocol": {
          "only": "udp"
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
  "settings_version": 4
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
