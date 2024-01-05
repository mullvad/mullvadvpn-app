use std::net::SocketAddr;

use super::{Error, Result};
use mullvad_types::{
    relay_constraints::{BridgeConstraints, BridgeSettings as NewBridgeSettings, BridgeType},
    settings::SettingsVersion,
};
use serde::{Deserialize, Serialize};
use talpid_types::net::{
    proxy::{CustomProxy, Shadowsocks, Socks5Local, Socks5Remote, SocksAuth},
    Endpoint, TransportProtocol,
};

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

/// Specifies a specific endpoint or [`BridgeConstraints`] to use when `mullvad-daemon` selects a
/// bridge server.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeSettings {
    /// Let the relay selection algorithm decide on bridges, based on the relay list.
    Normal(BridgeConstraints),
    Custom(ProxySettings),
}

/// Proxy server options to be used by `OpenVpnMonitor` when starting a tunnel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxySettings {
    Local(LocalProxySettings),
    Remote(RemoteProxySettings),
    Shadowsocks(ShadowsocksProxySettings),
}

/// Options for a generic proxy running on localhost.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalProxySettings {
    pub port: u16,
    pub peer: SocketAddr,
}

/// Options for a generic proxy running on remote host.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RemoteProxySettings {
    pub address: SocketAddr,
    pub auth: Option<ProxyAuth>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

/// Options for a bundled Shadowsocks proxy.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ShadowsocksProxySettings {
    pub peer: SocketAddr,
    /// Password on peer.
    pub password: String,
    pub cipher: String,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
}

// ======================================================
/// This is a closed migration.

/// We change bridge settings to no longer be an enum with custom and normal variants. It now is a
/// struct which contains a bridge type, a normal relay constraint and optional custom constraints.
///
/// We also migrate api access methods to use a slightly more shallow CustomProxy implementation
/// that instead of having a Socks5 and Shadowsocks variant instead has a Socks5Local, Socks5Remote
/// and Shadowsocks variant.
///
/// We also take the oppertunity to rename a couple of fields that relate to proxy types.
/// We rename
/// - shadowsocks.peer to shadowsocks.endpoint
/// - socks5_remote.authentication to socks5_remote.auth
/// - socks5_remote.peer to socks5_remote.endpoint
pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V8");

    migrate_bridge_settings(settings)?;

    migrate_api_access_settings(settings)?;

    settings["settings_version"] = serde_json::json!(SettingsVersion::V8);

    Ok(())
}

fn migrate_api_access_settings(settings: &mut serde_json::Value) -> Result<()> {
    if let Some(access_method_settings_list) = settings
        .get_mut("api_access_methods")
        .and_then(|api_access_methods| api_access_methods.get_mut("access_method_settings"))
        .and_then(|access_method_settings| access_method_settings.as_array_mut())
    {
        for access_method_setting in access_method_settings_list {
            let access_method = access_method_setting
                .get_mut("access_method")
                .ok_or(Error::InvalidSettingsContent)?;
            match access_method.get_mut("custom") {
                None => continue,
                Some(custom_access_method) => {
                    if let Some(shadowsocks) = custom_access_method.get_mut("shadowsocks") {
                        rename_field(shadowsocks, "peer", "endpoint")?;
                    } else if let Some(new_socks5_local) = custom_access_method
                        .get("socks5")
                        .and_then(|socks5| socks5.get("local"))
                    {
                        custom_access_method["socks5_local"] = new_socks5_local.clone();
                        custom_access_method
                            .as_object_mut()
                            .ok_or(Error::InvalidSettingsContent)?
                            .remove("socks5");
                    } else if let Some(socks5_remote) = custom_access_method
                        .get("socks5")
                        .and_then(|socks5| socks5.get("remote"))
                    {
                        let mut new_socks5_remote = socks5_remote.clone();
                        rename_field(&mut new_socks5_remote, "authentication", "auth")?;
                        rename_field(&mut new_socks5_remote, "peer", "endpoint")?;

                        custom_access_method["socks5_remote"] = new_socks5_remote;
                        custom_access_method
                            .as_object_mut()
                            .ok_or(Error::InvalidSettingsContent)?
                            .remove("socks5");
                    } else {
                        return Err(Error::InvalidSettingsContent);
                    }
                }
            }
        }
    }

    Ok(())
}

fn migrate_bridge_settings(settings: &mut serde_json::Value) -> Result<()> {
    let new = if let Some(custom_bridge_local) = settings
        .get_mut("bridge_settings")
        .and_then(|bridge_settings| bridge_settings.get_mut("custom"))
        .and_then(|bridge_settings_custom| bridge_settings_custom.get_mut("local"))
    {
        NewBridgeSettings {
            bridge_type: BridgeType::Custom,
            normal: BridgeConstraints::default(),
            custom: Some(CustomProxy::Socks5Local(Socks5Local {
                remote_endpoint: Endpoint {
                    address: extract_str(custom_bridge_local.get("peer"))?
                        .parse()
                        .map_err(|_| Error::InvalidSettingsContent)?,
                    protocol: TransportProtocol::Tcp,
                },
                local_port: custom_bridge_local
                    .get("port")
                    .ok_or(Error::InvalidSettingsContent)?
                    .as_u64()
                    .ok_or(Error::InvalidSettingsContent)?
                    .try_into()
                    .map_err(|_| Error::InvalidSettingsContent)?,
            })),
        }
    } else if let Some(custom_bridge_remote) = settings
        .get_mut("bridge_settings")
        .and_then(|bridge_settings| bridge_settings.get_mut("custom"))
        .and_then(|bridge_settings_custom| bridge_settings_custom.get_mut("remote"))
    {
        NewBridgeSettings {
            bridge_type: BridgeType::Custom,
            normal: BridgeConstraints::default(),
            custom: Some(CustomProxy::Socks5Remote(Socks5Remote {
                endpoint: extract_str(custom_bridge_remote.get("address"))?
                    .parse()
                    .map_err(|_| Error::InvalidSettingsContent)?,
                auth: custom_bridge_remote.get("auth").and_then(|auth| {
                    Some(SocksAuth {
                        username: auth.get("username")?.to_string(),
                        password: auth.get("password")?.to_string(),
                    })
                }),
            })),
        }
    } else if let Some(custom_bridge_shadowsocks) = settings
        .get_mut("bridge_settings")
        .and_then(|bridge_settings| bridge_settings.get_mut("custom"))
        .and_then(|bridge_settings_custom| bridge_settings_custom.get_mut("shadowsocks"))
    {
        NewBridgeSettings {
            bridge_type: BridgeType::Custom,
            normal: BridgeConstraints::default(),
            custom: Some(CustomProxy::Shadowsocks(Shadowsocks {
                endpoint: extract_str(custom_bridge_shadowsocks.get("peer"))?
                    .parse()
                    .map_err(|_| Error::InvalidSettingsContent)?,
                password: extract_str(custom_bridge_shadowsocks.get("password"))?.to_string(),
                cipher: extract_str(custom_bridge_shadowsocks.get("cipher"))?.to_string(),
            })),
        }
    } else if let Some(normal_bridge) = settings
        .get_mut("bridge_settings")
        .and_then(|bridge_settings| bridge_settings.get_mut("normal"))
    {
        NewBridgeSettings {
            bridge_type: BridgeType::Normal,
            normal: serde_json::from_value(normal_bridge.clone()).map_err(Error::Serialize)?,
            custom: None,
        }
    } else {
        return Ok(());
    };

    settings["bridge_settings"] = serde_json::json!(new);

    Ok(())
}

fn extract_str(opt: Option<&serde_json::Value>) -> Result<&str> {
    opt.ok_or(Error::InvalidSettingsContent)?
        .as_str()
        .ok_or(Error::InvalidSettingsContent)
}

fn rename_field(object: &mut serde_json::Value, old_name: &str, new_name: &str) -> Result<()> {
    object[new_name] = object
        .get(old_name)
        .ok_or(Error::InvalidSettingsContent)?
        .clone();
    object
        .as_object_mut()
        .ok_or(Error::InvalidSettingsContent)?
        .remove(old_name);
    Ok(())
}

fn version_matches(settings: &mut serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V7 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use crate::migrations::v7::{migrate_api_access_settings, migrate_bridge_settings};

    use super::{migrate, version_matches};

    pub const V7_SETTINGS: &str = r#"
{

  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": [
              "ch",
              "zrh",
              "ch-zrh-ovpn-001"
            ]
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
    "custom": {
      "local": {
        "port": 1080,
        "peer": "1.3.3.7:22"
      }
    }
  },
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "8cbdcfc8-fa7b-41de-8d12-26fa37439f89",
        "name": "Direct",
        "enabled": true,
        "access_method": {
          "built_in": "direct"
        }
      },
      {
        "id": "1d0d8891-dbb3-4439-a8f7-0e7d742ddbe4",
        "name": "Mullvad Bridges",
        "enabled": true,
        "access_method": {
          "built_in": "bridge"
        }
      },
      {
        "id": "1aaff7ab-e09f-4c03-af02-765e41943a7b",
        "name": "localsox",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5": {
              "local": {
                "remote_endpoint": {
                  "address": "1.3.3.7:1080",
                  "protocol": "tcp"
                },
                "local_port": 1079
              }
            }
          }
        }
      },
      {
        "id": "1e377232-8a53-4414-8b8f-f487227aaedb",
        "name": "remotesox",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5": {
              "remote": {
                "peer": "1.3.3.7:1080",
                "authentication": null
              }
            }
          }
        }
      },
      {
        "id": "74e5c659-acdd-4cad-a632-a25bf63c20e2",
        "name": "remotess",
        "enabled": true,
        "access_method": {
          "custom": {
            "shadowsocks": {
              "peer": "1.3.3.7:1080",
              "password": "mypass",
              "cipher": "aes-128-cfb"
            }
          }
        }
      }
    ]
  },
  "obfuscation_settings": {
    "selected_obfuscation": "udp2_tcp",
    "udp2tcp": {
      "port": "any"
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
      },
      "quantum_resistant": "auto"
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
  "settings_version": 7
}
"#;

    pub const V8_SETTINGS: &str = r#"
{

  "relay_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "hostname": [
              "ch",
              "zrh",
              "ch-zrh-ovpn-001"
            ]
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
    "bridge_type": "custom",
    "normal": {
        "location": "any",
        "providers": "any",
        "ownership": "any"
    },
    "custom": {
      "socks5_local": {
        "local_port": 1080,
        "remote_endpoint": {
            "address": "1.3.3.7:22",
            "protocol": "tcp"
        }
      }
    }
  },
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "8cbdcfc8-fa7b-41de-8d12-26fa37439f89",
        "name": "Direct",
        "enabled": true,
        "access_method": {
          "built_in": "direct"
        }
      },
      {
        "id": "1d0d8891-dbb3-4439-a8f7-0e7d742ddbe4",
        "name": "Mullvad Bridges",
        "enabled": true,
        "access_method": {
          "built_in": "bridge"
        }
      },
      {
        "id": "1aaff7ab-e09f-4c03-af02-765e41943a7b",
        "name": "localsox",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5_local": {
              "remote_endpoint": {
                "address": "1.3.3.7:1080",
                "protocol": "tcp"
              },
              "local_port": 1079
            }
          }
        }
      },
      {
        "id": "1e377232-8a53-4414-8b8f-f487227aaedb",
        "name": "remotesox",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5_remote": {
              "endpoint": "1.3.3.7:1080",
              "auth": null
            }
          }
        }
      },
      {
        "id": "74e5c659-acdd-4cad-a632-a25bf63c20e2",
        "name": "remotess",
        "enabled": true,
        "access_method": {
          "custom": {
            "shadowsocks": {
              "endpoint": "1.3.3.7:1080",
              "password": "mypass",
              "cipher": "aes-128-cfb"
            }
          }
        }
      }
    ]
  },
  "obfuscation_settings": {
    "selected_obfuscation": "udp2_tcp",
    "udp2tcp": {
      "port": "any"
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
      },
      "quantum_resistant": "auto"
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
  "settings_version": 8
}
"#;

    #[test]
    fn test_v7_to_v8_migration() {
        let mut old_settings = serde_json::from_str(V7_SETTINGS).unwrap();

        assert!(version_matches(&mut old_settings));
        migrate(&mut old_settings).unwrap();
        let new_settings: serde_json::Value = serde_json::from_str(V8_SETTINGS).unwrap();

        assert_eq!(&old_settings, &new_settings);
    }

    #[test]
    fn test_bridge_settings_custom_local_proxy() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "custom": {
      "local": {
        "port": 1080,
        "peer": "1.3.3.7:22"
      }
    }
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "bridge_type": "custom",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": {
      "socks5_local": {
        "local_port": 1080,
        "remote_endpoint": {
            "address": "1.3.3.7:22",
            "protocol": "tcp"
        }
      }
    }
  }
}"#,
        )
        .unwrap();

        migrate_bridge_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_bridge_settings_custom_remote_proxy() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "custom": {
      "remote": {
        "address": "1.3.3.7:1080",
        "auth": null
      }
    }
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "bridge_type": "custom",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": {
      "socks5_remote": {
        "endpoint": "1.3.3.7:1080",
        "auth": null
      }
    }
  }
}"#,
        )
        .unwrap();

        migrate_bridge_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_bridge_settings_custom_shadowsocks_proxy() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "custom": {
      "shadowsocks": {
        "peer": "1.3.3.7:1080",
        "password": "mypass",
        "cipher": "aes-128-cfb",
        "fwmark": 1836018789
      }
    }
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "bridge_type": "custom",
    "normal": {
      "location": "any",
      "providers": "any",
      "ownership": "any"
    },
    "custom": {
      "shadowsocks": {
        "endpoint": "1.3.3.7:1080",
        "password": "mypass",
        "cipher": "aes-128-cfb"
      }
    }
  }
}"#,
        )
        .unwrap();
        migrate_bridge_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_bridge_settings_normal() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any"
    }
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  }
}"#,
        )
        .unwrap();
        migrate_bridge_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_bridge_settings_specific_location() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any"
    }
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "bridge_settings": {
    "bridge_type": "normal",
    "normal": {
      "location": {
        "only": {
          "location": {
            "country": "se"
          }
        }
      },
      "providers": "any",
      "ownership": "any"
    },
    "custom": null
  }
}"#,
        )
        .unwrap();
        migrate_bridge_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_api_access_methods_custom_socks5_local() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "5eb9b2ee-f764-47c8-8111-ee95910d0099",
        "name": "mysocks",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5": {
              "local": {
                "remote_endpoint": {
                  "address": "1.3.3.7:22",
                  "protocol": "tcp"
                },
                "local_port": 1080
              }
            }
          }
        }
      }
    ]
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "5eb9b2ee-f764-47c8-8111-ee95910d0099",
        "name": "mysocks",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5_local": {
              "remote_endpoint": {
                "address": "1.3.3.7:22",
                "protocol": "tcp"
              },
              "local_port": 1080
            }
          }
        }
      }
    ]
  }
}"#,
        )
        .unwrap();

        migrate_api_access_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_api_access_methods_custom_socks5_remote() {
        println!("wew");
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "8e377232-8a53-4414-8b8f-f487227aaedb",
        "name": "remotesox",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5": {
              "remote": {
                "peer": "1.3.3.7:1080",
                "authentication": null
              }
            }
          }
        }
      }
    ]
  }
}"#,
        )
        .unwrap();
        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "8e377232-8a53-4414-8b8f-f487227aaedb",
        "name": "remotesox",
        "enabled": false,
        "access_method": {
          "custom": {
            "socks5_remote": {
              "endpoint": "1.3.3.7:1080",
              "auth": null
            }
          }
        }
      }
    ]
  }
}"#,
        )
        .unwrap();

        migrate_api_access_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }

    #[test]
    fn test_api_access_methods_custom_socks5_shadowsocks() {
        let mut pre: serde_json::Value = serde_json::from_str(
            r#"
{
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "74e5c659-acdd-4cad-a632-a25bf63c20e2",
        "name": "remotess",
        "enabled": true,
        "access_method": {
          "custom": {
            "shadowsocks": {
              "peer": "1.3.3.7:1080",
              "password": "mypass",
              "cipher": "aes-128-cfb"
            }
          }
        }
      }
    ]
  }
}"#,
        )
        .unwrap();

        let post: serde_json::Value = serde_json::from_str(
            r#"
{
  "api_access_methods": {
    "access_method_settings": [
      {
        "id": "74e5c659-acdd-4cad-a632-a25bf63c20e2",
        "name": "remotess",
        "enabled": true,
        "access_method": {
          "custom": {
            "shadowsocks": {
              "endpoint": "1.3.3.7:1080",
              "password": "mypass",
              "cipher": "aes-128-cfb"
            }
          }
        }
      }
    ]
  }
}"#,
        )
        .unwrap();

        migrate_api_access_settings(&mut pre).unwrap();
        assert_eq!(pre, post);
    }
}
