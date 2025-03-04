use serde::{Deserialize, Serialize};
#[cfg(target_os = "android")]
use serde_json::json;
#[cfg(target_os = "android")]
use std::{
    fs::{read_to_string, remove_file},
    path::Path,
};

use mullvad_types::settings::SettingsVersion;

use super::{Error, Result};

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.
type JsonSettings = serde_json::Map<String, serde_json::Value>;

/// Directories which the migration may want to touch.
#[cfg(target_os = "android")]
pub struct Directories<'path> {
    /// The path to the directory where `settings.json` is stored.
    pub settings: &'path Path,
}

/// The file where all currently split-tunnelled apps are stored.
#[cfg(target_os = "android")]
const SPLIT_TUNNELING_APPS: &str = "split-tunnelling.txt";
/// The file where the split-tunnelling state (enabled / disabled) is stored.
#[cfg(target_os = "android")]
const SPLIT_TUNNELING_STATE: &str = "split-tunnelling-enabled.txt";

/// Tunnel protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "tunnel_type")]
pub enum TunnelType {
    #[serde(rename = "openvpn")]
    OpenVpn,
    #[serde(rename = "wireguard")]
    Wireguard,
}

// ======================================================

/// This is an open migration
///
/// This migration onboards the Android app's split tunnel settings into the daemon's settings.
///
/// Until now, split tunneling has been completely handled client side by the Android app. This
/// includes keeping track of the setting itself (enabled / disabled) as well as all apps whose
/// traffic is supposed to be routed outside of any active tunnel. This migration reads all split
/// apps which has been stored by the Android client and writes them to the daemon's settings,
/// adding the 'split_tunnel' key to the settings object in the process.
///
/// # Note
/// This `migrate` function needs to get passed a `settings_dir` to work on Android. This is
/// because the Android client will pass the settings directory when initializing the daemon,
/// which means that we can not know ahead of time where the settings are stored.
#[allow(unused_variables)]
pub fn migrate(
    settings: &mut serde_json::Value,
    #[cfg(target_os = "android")] directories: Option<Directories<'_>>,
) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V10");

    let json_blob = to_settings_object(settings)?;

    // TODO: Remove this comment when closing the migration:
    // While this is an open migration, we check to see if the split tunnel apps have been migrated
    // already. If so, we don't want to run the migration code again. The call to
    // `split_tunnel_subkey_exists` can safely be removed when closing this migration.
    #[cfg(target_os = "android")]
    if !android::split_tunnel_subkey_exists(json_blob) {
        if let Some(directories) = directories {
            android::migrate_split_tunnel_settings(json_blob, directories)?;
        } else {
            log::warn!(
                "Did not migrate old split tunnelled apps due to missing settings directory"
            );
        }
    }

    migrate_tunnel_type(settings)?;

    // TODO: Uncomment this when closing the migration:
    // json_blob["settings_version"] = serde_json::json!(SettingsVersion::V10);

    Ok(())
}

fn migrate_tunnel_type(settings: &mut serde_json::Value) -> Result<()> {
    let Some(ref mut normal) = relay_settings(settings) else {
        return Ok(());
    };
    match normal.get_mut("tunnel_protocol") {
        // Already migrated
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
        // Migrate
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
    }
    Ok(())
}

fn relay_settings(settings: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    settings.get_mut("relay_settings")?.get_mut("normal")
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V9 as u64)
        .unwrap_or(false)
}

/// Represent the settings blob for what it is: A JSON-object.
fn to_settings_object(settings: &mut serde_json::Value) -> Result<&mut JsonSettings> {
    settings
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)
}

#[cfg(target_os = "android")]
mod android {
    use super::*;

    /// Check if the "split_tunnel" subkey already exists on the settings blob.
    /// On Android, this key *should not* exist before this migration.
    pub fn split_tunnel_subkey_exists(settings: &mut JsonSettings) -> bool {
        settings.get("split_tunnel").is_some()
    }

    /// Read the existing split-tunneling settings which the Android client has kept track off and
    /// write them to the settings object.
    pub fn migrate_split_tunnel_settings(
        settings: &mut JsonSettings,
        directories: Directories<'_>,
    ) -> Result<()> {
        // Read the split tunnel state (enabled / disabled) & all split apps
        // If both files can not be read for whatever reason we should not migrate any actual data.
        // Instead, we fill in conservative default values instead.
        let (enabled, split_apps) = match (
            read_split_tunnel_state(&directories),
            read_split_apps(&directories),
        ) {
            (Some(enabled), Some(apps)) => (enabled, apps),
            _ => (false, vec![]),
        };

        // Write the split tunnel settings to the settings object.
        add_split_tunneling_settings(settings, enabled, split_apps);

        // Remove the old leftover settings files.
        remove_old_split_tunneling_directories(&directories);

        Ok(())
    }

    /// Add the "split_tunnel" subkey to the settings object while setting it's own subkeys to
    /// `enabled` and `apps`.
    pub fn add_split_tunneling_settings(
        settings: &mut JsonSettings,
        enabled: bool,
        apps: Vec<String>,
    ) {
        // Create the "split_tunnel" key in the settings object and store the read split tunnel
        // state in the daemon's settings
        settings.insert(
            "split_tunnel".to_string(),
            json!({ "enable_exclusions": enabled, "apps": apps }),
        );
    }

    /// Read the target file and parse the stored split tunneling state. If split tunneling was
    /// previously enabled in the android app, the return value of this function will be
    /// `Some(true)`, otherwise `Some(false)`.
    ///
    /// If the file could not be found or read, some logging will occur and `None` will be returned.
    pub fn read_split_tunnel_state(directories: &Directories<'_>) -> Option<bool> {
        let path = directories.settings.join(SPLIT_TUNNELING_STATE);
        log::trace!("Reading split tunnel state from {}", path.display());
        let enabled = read_to_string(path.clone())
            .inspect_err(|_| {
                log::error!("Could not read split tunnel state from {}", path.display())
            })
            .ok()?
            .trim()
            .eq("true");
        Some(enabled)
    }

    /// Read the target file and parse the stored split tunneled apps.
    ///
    /// If the file could not be found or read, some logging will occur and `None` will be returned.
    pub fn read_split_apps(directories: &Directories<'_>) -> Option<Vec<String>> {
        let path = directories.settings.join(SPLIT_TUNNELING_APPS);
        log::trace!("Reading split tunnel apps from {}", path.display());
        let split_apps = read_to_string(path.clone())
            .inspect_err(|_| {
                log::error!("Could not read split tunnel apps from {}", path.display())
            })
            .ok()?
            .lines()
            .map(str::to_owned)
            .collect();
        Some(split_apps)
    }

    /// Remove the lingering, old files split tunnelling related files. They should have been
    /// completely migrated to the daemon settings at this point, so they won't be needed any
    /// longer.
    ///
    /// Note: We don't really care if these operations fail - they won't ever be read again, and new
    /// app installations shall not create them.
    pub fn remove_old_split_tunneling_directories(directories: &Directories<'_>) {
        remove_file(directories.settings.join(SPLIT_TUNNELING_STATE))
            .inspect_err(|error| log::error!("Failed to remove {SPLIT_TUNNELING_STATE}: {error}"))
            .ok();
        remove_file(directories.settings.join(SPLIT_TUNNELING_APPS))
            .inspect_err(|error| log::error!("Failed to remove {SPLIT_TUNNELING_APPS}: {error}"))
            .ok();
    }
}

#[cfg(test)]
mod test {
    use super::{migrate, version_matches};

    #[cfg(target_os = "android")]
    mod android {
        /// Assert that split-tunneling settings has been added to the android settings post-migration.
        #[test]
        fn test_v9_to_v10_migration() {
            use crate::migrations::v9::{
                add_split_tunneling_settings,
                test::constants::{V10_ANDROID_SETTINGS, V9_ANDROID_SETTINGS},
            };

            let enabled = true;
            let apps = ["com.android.chrome", "net.mullvad.mullvadvpn"];

            let mut settings = serde_json::from_str(V9_ANDROID_SETTINGS).unwrap();
            // Perform the actual settings migration while skipping the I/O performed in
            // `migrate_split_tunnel_settings`.
            add_split_tunneling_settings(settings, enabled, apps);
            let new_settings = serde_json::from_str(V10_ANDROID_SETTINGS).unwrap();
            assert_eq!(settings, new_settings);
        }

        mod constants {
            /// This settings blob does not contain the "split_tunnel" option.
            pub const V9_ANDROID_SETTINGS: &str = r#"
  {
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": "at-vie-ovpn-001"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": false,
        "entry_location": {
          "only": {
            "location": {
              "country": "se"
            }
          }
        }
      },
      "openvpn_constraints": {
        "port": "any"
      }
    }
  },
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  },
  "obfuscation_settings": {
    "selected_obfuscation": "auto",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "custom_lists": {
    "custom_lists": []
  },
  "api_access_methods": {
    "direct": {
      "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "92135711-534d-4950-963d-93e446a792e4",
      "name": "Mullvad Bridges",
      "enabled": true,
      "access_method": {
        "built_in": "bridge"
      }
    },
    "custom": []
  },
  "allow_lan": false,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "quantum_resistant": "auto",
      "rotation_interval": null
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false,
        "block_malware": false,
        "block_adult_content": false,
        "block_gambling": false,
        "block_social_media": false
      },
      "custom_options": {
        "addresses": []
      }
    }
  },
  "relay_overrides": [],
  "show_beta_releases": true,
  "settings_version": 9
  }
  "#;

            /// This settings blob *should* contain the "split_tunnel" option.
            pub const V10_ANDROID_SETTINGS: &str = r#"
  {
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "wireguard",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": false,
        "entry_location": {
          "only": {
            "location": {
              "country": "se"
            }
          }
        }
      },
      "openvpn_constraints": {
        "port": "any"
      }
    }
  },
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  },
  "obfuscation_settings": {
    "selected_obfuscation": "auto",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "custom_lists": {
    "custom_lists": []
  },
  "api_access_methods": {
    "direct": {
      "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "92135711-534d-4950-963d-93e446a792e4",
      "name": "Mullvad Bridges",
      "enabled": true,
      "access_method": {
        "built_in": "bridge"
      }
    },
    "custom": []
  },
  "allow_lan": false,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "quantum_resistant": "auto",
      "rotation_interval": null
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false,
        "block_malware": false,
        "block_adult_content": false,
        "block_gambling": false,
        "block_social_media": false
      },
      "custom_options": {
        "addresses": []
      }
    }
  },
  "relay_overrides": [],
  "show_beta_releases": true,
  "split_tunnel": {
    "enable_exclusions": true,
    "apps": ["com.android.chrome", "net.mullvad.mullvadvpn"]
  }
  "settings_version": 10
  }
  "#;
        }
    }

    /// Assert that tunnel type is migrated
    #[test]
    fn test_v9_to_v10_migration() {
        let mut old_settings = serde_json::from_str(V9_SETTINGS).unwrap();

        assert!(version_matches(&old_settings));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V10_SETTINGS).unwrap();

        eprintln!(
            "old_settings: {}",
            serde_json::to_string_pretty(&old_settings).unwrap()
        );
        eprintln!(
            "new_settings: {}",
            serde_json::to_string_pretty(&new_settings).unwrap()
        );

        assert_eq!(&old_settings, &new_settings);
    }

    /// This settings blob contains no constraint for tunnel type
    pub const V9_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": "at-vie-ovpn-001"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "any",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": false,
        "entry_location": {
          "only": {
            "location": {
              "country": "se"
            }
          }
        }
      },
      "openvpn_constraints": {
        "port": "any"
      }
    }
  },
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  },
  "obfuscation_settings": {
    "selected_obfuscation": "auto",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "custom_lists": {
    "custom_lists": []
  },
  "api_access_methods": {
    "direct": {
      "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "92135711-534d-4950-963d-93e446a792e4",
      "name": "Mullvad Bridges",
      "enabled": true,
      "access_method": {
        "built_in": "bridge"
      }
    },
    "custom": []
  },
  "allow_lan": false,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "quantum_resistant": "auto",
      "rotation_interval": null
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false,
        "block_malware": false,
        "block_adult_content": false,
        "block_gambling": false,
        "block_social_media": false
      },
      "custom_options": {
        "addresses": []
      }
    }
  },
  "relay_overrides": [],
  "show_beta_releases": true,
  "settings_version": 9
}
"#;

    /// This settings blob does not contain an "any" tunnel type
    pub const V10_SETTINGS: &str = r#"
{
  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": "at-vie-ovpn-001"
          }
        }
      },
      "providers": "any",
      "ownership": "any",
      "tunnel_protocol": "openvpn",
      "wireguard_constraints": {
        "port": "any",
        "ip_version": "any",
        "use_multihop": false,
        "entry_location": {
          "only": {
            "location": {
              "country": "se"
            }
          }
        }
      },
      "openvpn_constraints": {
        "port": "any"
      }
    }
  },
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  },
  "obfuscation_settings": {
    "selected_obfuscation": "auto",
    "udp2tcp": {
      "port": "any"
    }
  },
  "bridge_state": "auto",
  "custom_lists": {
    "custom_lists": []
  },
  "api_access_methods": {
    "direct": {
      "id": "d81121bf-c942-4ca4-971f-8ea6581bc915",
      "name": "Direct",
      "enabled": true,
      "access_method": {
        "built_in": "direct"
      }
    },
    "mullvad_bridges": {
      "id": "92135711-534d-4950-963d-93e446a792e4",
      "name": "Mullvad Bridges",
      "enabled": true,
      "access_method": {
        "built_in": "bridge"
      }
    },
    "custom": []
  },
  "allow_lan": false,
  "block_when_disconnected": false,
  "auto_connect": false,
  "tunnel_options": {
    "openvpn": {
      "mssfix": null
    },
    "wireguard": {
      "mtu": null,
      "quantum_resistant": "auto",
      "rotation_interval": null
    },
    "generic": {
      "enable_ipv6": false
    },
    "dns_options": {
      "state": "default",
      "default_options": {
        "block_ads": false,
        "block_trackers": false,
        "block_malware": false,
        "block_adult_content": false,
        "block_gambling": false,
        "block_social_media": false
      },
      "custom_options": {
        "addresses": []
      }
    }
  },
  "relay_overrides": [],
  "show_beta_releases": true,
  "settings_version": 9
}
"#;
}
