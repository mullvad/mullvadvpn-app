use super::Result;
use mullvad_types::{constraints::Constraint, settings::SettingsVersion};
use serde::{Deserialize, Serialize};

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

/// The tunnel protocol used by a [`TunnelEndpoint`][talpid_types::net::TunnelEndpoint].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "tunnel_type")]
pub enum TunnelType {
    #[serde(rename = "openvpn")]
    OpenVpn,
    #[serde(rename = "wireguard")]
    Wireguard,
}

// ======================================================

pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    let version_matches = |settings: &serde_json::Value| settings.get("settings_version").is_none();
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

    if let Some(relay_settings) = settings.get_mut("relay_settings")
        && let Some(normal_settings) = relay_settings.get_mut("normal")
    {
        if let Some(openvpn_constraints) = openvpn_constraints {
            normal_settings["openvpn_constraints"] = openvpn_constraints;
            normal_settings["tunnel_protocol"] = serde_json::json!(Constraint::<TunnelType>::Any);
        } else if let Some(wireguard_constraints) = wireguard_constraints {
            normal_settings["wireguard_constraints"] = wireguard_constraints;
            normal_settings["tunnel_protocol"] =
                serde_json::json!(Constraint::Only(TunnelType::Wireguard));
        } else {
            normal_settings["tunnel_protocol"] = serde_json::json!(Constraint::<TunnelType>::Any);
        }
        if let Some(object) = normal_settings.as_object_mut() {
            object.remove("tunnel");
        }
    }

    settings["show_beta_releases"] = serde_json::json!(false);
    settings["settings_version"] = serde_json::json!(SettingsVersion::V2);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::migrations::load_seed;

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

    const V1_SETTINGS: &str = include_str!("v1_settings.json");

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
    fn v1_to_v2_migration() {
        let mut settings = load_seed("v1.json");
        migrate(&mut settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings).unwrap());
    }

    #[test]
    fn v1_2019v3_to_v2_migration() {
        let mut settings = load_seed("v1_2019v3.json");
        migrate(&mut settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings).unwrap());
    }
}
