use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

pub type AccountToken = String;

// ======================================================

pub(crate) struct MigrationData {
    pub token: AccountToken,
    pub wg_data: Option<serde_json::Value>,
}

/// This is an open ended migration. There is no v6 yet!
/// The migrations performed by this function are still backwards compatible.
/// The JSON coming out of this migration can be read by any v5 compatible daemon.
///
/// When further migrations are needed, add them here and if they are not backwards
/// compatible then create v6 and "close" this migration for further modification.
///
/// # Changes to the format
///
/// The ability to disable WireGuard multihop while preserving the entry location was added.
/// So a new field, `use_multihop` is introduced. We want this to default to `true` iff:
///  * `use_mulithop` was not present in the settings
///  * A multihop entry location had been previously specified.
///
/// It is also no longer valid to have `entry_location` set to null. So remove the field if it
/// is null in order to make it default back to the default location.
///
/// This also removes the account token and WireGuard key from the settings.
pub(crate) async fn migrate(settings: &mut serde_json::Value) -> Result<Option<MigrationData>> {
    if !version_matches(settings) {
        return Ok(None);
    }
    let wireguard_constraints = || -> Option<&serde_json::Value> {
        settings
            .get("relay_settings")?
            .get("normal")?
            .get("wireguard_constraints")
    }();
    if let Some(constraints) = wireguard_constraints {
        if let Some(location) = constraints.get("entry_location") {
            if constraints.get("use_multihop").is_none() {
                if location.is_null() {
                    // "Null" is no longer valid. It is not an option.
                    settings["relay_settings"]["normal"]["wireguard_constraints"]
                        .as_object_mut()
                        .ok_or(Error::NoMatchingVersion)?
                        .remove("entry_location");
                } else {
                    settings["relay_settings"]["normal"]["wireguard_constraints"]["use_multihop"] =
                        serde_json::json!(true);
                }
            }
        }
    }

    if let Some(token) = settings.get("account_token").filter(|t| !t.is_null()) {
        let token: AccountToken =
            serde_json::from_value(token.clone()).map_err(Error::ParseError)?;
        let migration_data =
            if let Some(wg_data) = settings.get("wireguard").filter(|wg| !wg.is_null()) {
                Ok(Some(MigrationData {
                    token,
                    wg_data: Some(wg_data.clone()),
                }))
            } else {
                Ok(Some(MigrationData {
                    token,
                    wg_data: None,
                }))
            };

        let settings_map = settings.as_object_mut().ok_or(Error::NoMatchingVersion)?;
        settings_map.remove("account_token");
        settings_map.remove("wireguard");

        return migration_data;
    }

    // Note: Not incrementing the version number yet, since this migration is still open
    // for future modification.
    // settings["settings_version"] = serde_json::json!(SettingsVersion::V6);

    Ok(None)
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V5 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::{migrate, version_matches};
    use serde_json;

    pub const V5_SETTINGS_V1: &str = r#"
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
        "port": "any",
        "ip_version": "any",
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

    pub const V5_SETTINGS_V2: &str = r#"
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

    #[tokio::test]
    async fn test_v5_v1_migration() {
        let mut old_settings = serde_json::from_str(V5_SETTINGS_V1).unwrap();

        assert!(version_matches(&mut old_settings));
        migrate(&mut old_settings).await.unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V5_SETTINGS_V2).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }
}
