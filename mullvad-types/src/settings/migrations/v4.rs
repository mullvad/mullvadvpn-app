use super::{Error, Result, SettingsVersion};
use crate::relay_constraints::{Constraint, TransportPort};
use talpid_types::net::TransportProtocol;


const WIREGUARD_TCP_PORTS: [u16; 3] = [80, 443, 5001];
const OPENVPN_TCP_PORTS: [u16; 2] = [80, 443];


pub(super) struct Migration;

impl super::SettingsMigration for Migration {
    fn version_matches(&self, settings: &mut serde_json::Value) -> bool {
        settings
            .get("settings_version")
            .map(|version| version == SettingsVersion::V4 as u64)
            .unwrap_or(false)
    }

    fn migrate(&self, settings: &mut serde_json::Value) -> Result<()> {
        log::info!("Migrating settings format to V5");

        let wireguard_constraints = || -> Option<&serde_json::Value> {
            settings
                .get("relay_settings")?
                .get("normal")?
                .get("wireguard_constraints")
        }();

        if let Some(constraints) = wireguard_constraints {
            let (port, protocol): (Constraint<u16>, TransportProtocol) = if let Some(port) =
                constraints.get("port")
            {
                let port_constraint =
                    serde_json::from_value(port.clone()).map_err(Error::ParseError)?;
                match port_constraint {
                    Constraint::Any => (Constraint::Any, TransportProtocol::Udp),
                    Constraint::Only(port) => (Constraint::Only(port), wg_protocol_from_port(port)),
                }
            } else {
                (Constraint::Any, TransportProtocol::Udp)
            };

            settings["relay_settings"]["normal"]["wireguard_constraints"]["port"] = match port {
                Constraint::Any => {
                    serde_json::json!(Constraint::<TransportPort>::Any)
                }
                Constraint::Only(_) => {
                    serde_json::json!(Constraint::Only(TransportPort { protocol, port }))
                }
            };

            settings["relay_settings"]["normal"]["wireguard_constraints"]
                .as_object_mut()
                .ok_or(Error::NoMatchingVersion)?
                .remove("protocol");
        }

        let openvpn_constraints = || -> Option<&serde_json::Value> {
            settings
                .get("relay_settings")?
                .get("normal")?
                .get("openvpn_constraints")
        }();

        if let Some(constraints) = openvpn_constraints {
            let port: Constraint<u16> = if let Some(port) = constraints.get("port") {
                serde_json::from_value(port.clone()).map_err(Error::ParseError)?
            } else {
                Constraint::Any
            };
            let transport_constraint: Constraint<TransportProtocol> =
                if let Some(protocol) = constraints.get("protocol") {
                    serde_json::from_value(protocol.clone()).map_err(Error::ParseError)?
                } else {
                    Constraint::Any
                };

            let port = match (port, transport_constraint) {
                (Constraint::Only(port), Constraint::Any) => Constraint::Only(TransportPort {
                    protocol: openvpn_protocol_from_port(port),
                    port: Constraint::Only(port),
                }),
                (Constraint::Only(port), Constraint::Only(protocol)) => {
                    Constraint::Only(TransportPort {
                        protocol,
                        port: Constraint::Only(port),
                    })
                }
                (Constraint::Any, Constraint::Only(protocol)) => Constraint::Only(TransportPort {
                    protocol,
                    port: Constraint::Any,
                }),
                (Constraint::Any, Constraint::Any) => Constraint::Any,
            };

            settings["relay_settings"]["normal"]["openvpn_constraints"]["port"] =
                serde_json::json!(port);
            settings["relay_settings"]["normal"]["openvpn_constraints"]
                .as_object_mut()
                .ok_or(Error::NoMatchingVersion)?
                .remove("protocol");
        }

        settings["settings_version"] = serde_json::json!(SettingsVersion::V5);

        Ok(())
    }
}

fn openvpn_protocol_from_port(port: u16) -> TransportProtocol {
    log::warn!("Inferring transport protocol from port constraint");
    if OPENVPN_TCP_PORTS.contains(&port) {
        TransportProtocol::Tcp
    } else {
        TransportProtocol::Udp
    }
}

fn wg_protocol_from_port(port: u16) -> TransportProtocol {
    log::warn!("Inferring transport protocol from port constraint");
    if WIREGUARD_TCP_PORTS.contains(&port) {
        TransportProtocol::Tcp
    } else {
        TransportProtocol::Udp
    }
}

#[cfg(test)]
mod test {
    use super::super::try_migrate_settings;
    use serde_json;

    pub const V4_SETTINGS: &str = r#"
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
        "port": {
          "only": 80
        },
        "protocol": "any"
      },
      "openvpn_constraints": {
        "port": {
          "only": 1195
        },
        "protocol": "any"
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
  "settings_version": 4
}
"#;

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
        "port": {
          "only": {
            "protocol": "tcp",
            "port": {
              "only": 80
            }
          }
        }
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
  "settings_version": 5
}
"#;


    #[test]
    fn test_v4_migration() {
        let migrated_settings =
            try_migrate_settings(V4_SETTINGS.as_bytes()).expect("Migration failed");
        let new_settings = serde_json::from_str(NEW_SETTINGS).unwrap();

        assert_eq!(&migrated_settings, &new_settings);
    }
}
