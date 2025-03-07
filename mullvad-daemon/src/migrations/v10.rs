use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;
use talpid_types::net::TunnelType;

/// Automatic tunnel protocol has been removed. If the tunnel protocol is set to `any`, it will be
/// migrated to `wireguard`, unless the location is an openvpn relay, in which case it will be
/// migrated to `openvpn`.
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !(version(settings) == Some(SettingsVersion::V10)) {
        return Ok(());
    }

    log::info!("Migrating settings format to v11");

    migrate_tunnel_type(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V11);

    Ok(())
}

fn version(settings: &serde_json::Value) -> Option<SettingsVersion> {
    settings
        .get("settings_version")
        .and_then(|version| serde_json::from_value(version.clone()).ok())
}

fn relay_settings(settings: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    settings.get_mut("relay_settings")?.get_mut("normal")
}

fn migrate_tunnel_type(settings: &mut serde_json::Value) -> Result<()> {
    if let Some(ref mut normal) = relay_settings(settings) {
        migrate_tunnel_type_inner(normal)?;
    }
    Ok(())
}

fn migrate_tunnel_type_inner(normal: &mut serde_json::Value) -> Result<()> {
    match normal.get_mut("tunnel_protocol") {
        // Migrate tunnel protocol "any"
        Some(serde_json::Value::String(s)) if s == "any" => {
            // If openvpn is selected, migrate to openvpn tunnel type
            // Otherwise, select wireguard
            let hostname = normal
                .get_mut("location")
                .and_then(|location| location.get_mut("only"))
                .and_then(|only| only.get_mut("location"))
                .and_then(|only| only.get_mut("hostname").cloned());

            let protocol = if let Some(serde_json::Value::String(s)) = hostname {
                if s.split('-').any(|token| token == "ovpn") {
                    TunnelType::OpenVpn
                } else {
                    TunnelType::Wireguard
                }
            } else {
                TunnelType::Wireguard
            };

            normal["tunnel_protocol"] = serde_json::json!(protocol);
        }
        // Migrate '"only": { "tunnel_protocol": $tunnel_protocol }'
        // to '"tunnel_protocol": $tunnel_protocol'
        Some(serde_json::Value::Object(ref mut constraint)) => {
            if let Some(tunnel_type) = constraint.get("only") {
                let tunnel_type: TunnelType = serde_json::from_value(tunnel_type.clone())
                    .map_err(|_| Error::InvalidSettingsContent)?;
                normal["tunnel_protocol"] = serde_json::json!(tunnel_type);
            } else {
                return Err(Error::InvalidSettingsContent);
            }
        }
        Some(_) => {
            return Err(Error::InvalidSettingsContent);
        }
        // Unexpected result. Do nothing.
        None => (),
    };
    Ok(())
}
#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::migrations::v10::migrate_tunnel_type_inner;

    /// Tunnel protocol "any" is migrated to wireguard
    #[test]
    fn test_v10_to_v11_migration() {
        let mut old_settings = json!({
            "tunnel_protocol": "any",
            "location": {
                "only": {
                    "location": {
                        "country": "se"
                    }
                }
            }
        });
        migrate_tunnel_type_inner(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = json!({
            "tunnel_protocol": "wireguard",
            "location": {
                "only": {
                    "location": {
                        "country": "se"
                    }
                }
            }
        });
        assert_eq!(&old_settings, &new_settings);
    }

    /// Tunnel protocol "any" is migrated to openvpn, since the location is an openvpn relay
    #[test]
    fn test_v10_to_v11_migration_openvpn_location() {
        let mut old_settings = json!({
            "tunnel_protocol": "any",
            "location": {
                "only": {
                    "location": {
                        "hostname": "at-vie-ovpn-001"
                    }
                }
            }
        });
        migrate_tunnel_type_inner(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = json!({
            "tunnel_protocol": "openvpn",
            "location": {
                "only": {
                    "location": {
                        "hostname": "at-vie-ovpn-001"
                    }
                }
            }
        });
        assert_eq!(&old_settings, &new_settings);
    }

    /// Tunnel protocol '"only" = { "tunnel_protocol" = '$tunnel_protocol' } is migrated to
    /// '"tunnel_protocol" = '$tunnel_protocol'
    #[test]
    fn test_v10_to_v11_migration_only_protocol() {
        let mut old_settings = json!({
            "tunnel_protocol": {
                "only": "wireguard"
            },
            "location": {
                "only": {
                    "location": {
                        "country": "se"
                    }
                }
            }
        });
        migrate_tunnel_type_inner(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = json!({
            "tunnel_protocol": "wireguard",
            "location": {
                "only": {
                    "location": {
                        "country": "se"
                    }
                }
            }
        });
        assert_eq!(&old_settings, &new_settings);
    }
}
