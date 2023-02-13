use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;
use std::net::IpAddr;

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DnsState {
    #[default]
    Default,
    Custom,
}

/// DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct DnsOptions {
    pub state: DnsState,
    pub default_options: DefaultDnsOptions,
    pub custom_options: CustomDnsOptions,
}

/// Default DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct DefaultDnsOptions {
    pub block_ads: bool,
    pub block_trackers: bool,
}

/// Custom DNS config
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomDnsOptions {
    pub addresses: Vec<IpAddr>,
}

// ======================================================

pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V4");

    let dns_options =
        || -> Option<&serde_json::Value> { settings.get("tunnel_options")?.get("dns_options") }();

    if let Some(options) = dns_options {
        if options.get("state").is_none() {
            let new_state = if options
                .get("custom")
                .map(|custom| custom.as_bool().unwrap_or(false))
                .unwrap_or(false)
            {
                DnsState::Custom
            } else {
                DnsState::Default
            };
            let addresses = if let Some(addrs) = options.get("addresses") {
                serde_json::from_value(addrs.clone()).map_err(|_| Error::InvalidSettingsContent)?
            } else {
                vec![]
            };

            settings["tunnel_options"]["dns_options"] = serde_json::json!(DnsOptions {
                state: new_state,
                default_options: DefaultDnsOptions::default(),
                custom_options: CustomDnsOptions { addresses },
            });
        }
    }

    settings["settings_version"] = serde_json::json!(SettingsVersion::V4);

    Ok(())
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V3 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::{migrate, version_matches};

    pub const V3_SETTINGS: &str = r#"
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
          "only": 1195
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
      "custom": false,
      "addresses": [
        "1.1.1.1",
        "1.2.3.4"
      ]
    }
  },
  "settings_version": 3
}
"#;

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
        "port": "any"
      },
      "openvpn_constraints": {
        "port": {
          "only": 1195
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

    #[test]
    fn test_v3_migration() {
        let mut old_settings = serde_json::from_str(V3_SETTINGS).unwrap();

        assert!(version_matches(&mut old_settings));

        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V4_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }
}
