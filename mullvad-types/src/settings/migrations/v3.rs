use super::{Error, Result, SettingsVersion};
use crate::settings::{CustomDnsOptions, DefaultDnsOptions, DnsOptions, DnsState};


pub(super) struct Migration;

impl super::SettingsMigration for Migration {
    fn version_matches(&self, settings: &mut serde_json::Value) -> bool {
        settings
            .get("settings_version")
            .map(|version| version == SettingsVersion::V3 as u64)
            .unwrap_or(false)
    }

    fn migrate(&self, settings: &mut serde_json::Value) -> Result<()> {
        log::info!("Migrating settings format to V4");

        let dns_options = || -> Option<&serde_json::Value> {
            settings.get("tunnel_options")?.get("dns_options")
        }();

        if let Some(options) = dns_options {
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
                serde_json::from_value(addrs.clone()).map_err(Error::ParseError)?
            } else {
                vec![]
            };

            settings["tunnel_options"]["dns_options"] = serde_json::json!(DnsOptions {
                state: new_state,
                default_options: DefaultDnsOptions::default(),
                custom_options: CustomDnsOptions { addresses },
            });
        }

        settings["settings_version"] = serde_json::json!(SettingsVersion::V4);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::try_migrate_settings;
    use serde_json;

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
        let migrated_settings =
            try_migrate_settings(V3_SETTINGS.as_bytes()).expect("Migration failed");
        let new_settings = serde_json::from_str(NEW_SETTINGS).unwrap();

        assert_eq!(&migrated_settings, &new_settings);
    }
}
