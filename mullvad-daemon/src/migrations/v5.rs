use super::{Error, Result};
use crate::{device::DeviceService, DaemonEventSender, InternalDaemonEvent};
use mullvad_types::{
    account::AccountToken, device::DeviceData, settings::SettingsVersion, wireguard::WireguardData,
};
use talpid_core::mpsc::Sender;
use talpid_types::ErrorExt;

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

// ======================================================

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
/// This also removes the account token and WireGuard key from the settings, looks up the
/// corresponding device, so that they can be stored in `device.json` instead. This is done by
/// sending the `DeviceMigrationEvent` event to the daemon.
pub(crate) async fn migrate(
    settings: &mut serde_json::Value,
    runtime: tokio::runtime::Handle,
    rest_handle: mullvad_rpc::rest::MullvadRestHandle,
    daemon_tx: DaemonEventSender,
) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    migrate_to_v5v2(settings)?;

    log::info!("Migrating settings format to V6");

    if let Some(token) = settings.get("account_token").filter(|t| !t.is_null()) {
        let api_handle = rest_handle.availability.clone();
        let service = DeviceService::new(rest_handle, api_handle);
        let token: AccountToken =
            serde_json::from_value(token.clone()).map_err(Error::ParseError)?;
        if let Some(wg_data) = settings.get("wireguard").filter(|wg| !wg.is_null()) {
            let wg_data: WireguardData =
                serde_json::from_value(wg_data.clone()).map_err(Error::ParseError)?;
            log::info!("Creating a new device cache from previous settings");
            runtime.spawn(cache_from_wireguard_key(daemon_tx, service, token, wg_data));
        } else {
            log::info!("Generating a new device for the account");
            runtime.spawn(cache_from_account(daemon_tx, service, token));
        }

        // TODO: Remove account token
        // TODO: Remove wireguard data
    }

    settings["settings_version"] = serde_json::json!(SettingsVersion::V6);

    Ok(())
}

fn migrate_to_v5v2(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
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
    Ok(())
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V5 as u64)
        .unwrap_or(false)
}

async fn cache_from_wireguard_key(
    daemon_tx: DaemonEventSender,
    service: DeviceService,
    token: AccountToken,
    wg_data: WireguardData,
) {
    let devices = match service.list_devices_with_backoff(token.clone()).await {
        Ok(devices) => devices,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to enumerate devices for account")
            );
            return;
        }
    };

    for device in devices.into_iter() {
        if device.pubkey == wg_data.private_key.public_key() {
            let _ = daemon_tx.send(InternalDaemonEvent::DeviceMigrationEvent(DeviceData {
                token,
                device,
                wg_data,
            }));
            return;
        }
    }
    log::info!("The existing WireGuard key is not valid; generating a new device");
    cache_from_account(daemon_tx, service, token).await;
}

async fn cache_from_account(
    daemon_tx: DaemonEventSender,
    service: DeviceService,
    token: AccountToken,
) {
    match service.generate_for_account_with_backoff(token).await {
        Ok(device_data) => {
            let _ = daemon_tx.send(InternalDaemonEvent::DeviceMigrationEvent(device_data));
        }
        Err(error) => log::error!(
            "{}",
            error.display_chain_with_msg("Failed to generate new device for account")
        ),
    }
}

#[cfg(test)]
mod test {
    use super::{migrate_to_v5v2, version_matches};
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

    #[test]
    fn test_v5_v1_migration() {
        let mut old_settings = serde_json::from_str(V5_SETTINGS_V1).unwrap();

        assert!(version_matches(&mut old_settings));

        migrate_to_v5v2(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V5_SETTINGS_V2).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }
}
