use super::{Error, Result};
use mullvad_types::{relay_constraints::Constraint, settings::SettingsVersion};

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

const WIREGUARD_TCP_PORTS: [u16; 3] = [80, 443, 5001];
const OPENVPN_TCP_PORTS: [u16; 2] = [80, 443];

/// Representation of a transport protocol, either UDP or TCP.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    /// Represents the UDP transport protocol.
    Udp,
    /// Represents the TCP transport protocol.
    Tcp,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct TransportPort {
    pub protocol: TransportProtocol,
    pub port: Constraint<u16>,
}

// ======================================================

pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V5");

    let wireguard_constraints = || -> Option<&serde_json::Value> {
        settings
            .get("relay_settings")?
            .get("normal")?
            .get("wireguard_constraints")
    }();

    if let Some(constraints) = wireguard_constraints {
        let (port, protocol): (Constraint<u16>, TransportProtocol) =
            if let Some(port) = constraints.get("port") {
                let port_constraint = serde_json::from_value(port.clone())
                    .map_err(|_| Error::InvalidSettingsContent)?;
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
            .ok_or(Error::InvalidSettingsContent)?
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
            serde_json::from_value(port.clone()).map_err(|_| Error::InvalidSettingsContent)?
        } else {
            Constraint::Any
        };
        let transport_constraint: Constraint<TransportProtocol> = if let Some(protocol) =
            constraints.get("protocol")
        {
            serde_json::from_value(protocol.clone()).map_err(|_| Error::InvalidSettingsContent)?
        } else {
            Constraint::Any
        };

        let port = match (port, transport_constraint) {
            (Constraint::Only(port), Constraint::Any) => Constraint::Only(TransportPort {
                protocol: openvpn_protocol_from_port(port),
                port: Constraint::Only(port),
            }),
            (port, Constraint::Only(protocol)) => {
                Constraint::Only(TransportPort { protocol, port })
            }
            (Constraint::Any, Constraint::Any) => Constraint::Any,
        };

        settings["relay_settings"]["normal"]["openvpn_constraints"]["port"] =
            serde_json::json!(port);
        settings["relay_settings"]["normal"]["openvpn_constraints"]
            .as_object_mut()
            .ok_or(Error::InvalidSettingsContent)?
            .remove("protocol");
    }

    settings["settings_version"] = serde_json::json!(SettingsVersion::V5);

    Ok(())
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V4 as u64)
        .unwrap_or(false)
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
    use super::{migrate, version_matches};

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

    pub const V5_SETTINGS: &str = r#"
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
        let mut old_settings = serde_json::from_str(V4_SETTINGS).unwrap();

        assert!(version_matches(&mut old_settings));

        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V5_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }
}
