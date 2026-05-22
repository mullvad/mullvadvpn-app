use super::Result;
use mullvad_types::settings::SettingsVersion;
use serde_json::{Value, json};
use talpid_types::net::proxy::ShadowsocksCipher;

/// NOTE: This migration has been closed.
///
/// This migration handles:
/// - Remove custom Shadowsocks api access methods containing unusable ciphers. This migration is
///   part of a bug fix which syncs the available Shadowsocks ciphers between the daemon and client
///   applications configuring custom access methods.
pub fn migrate(settings: &mut Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V17");

    discard_invalid_shadowsocks_access_methods(settings);

    settings["settings_version"] = json!(SettingsVersion::V17);

    Ok(())
}

/// { "api_access_methods": { "custom": [ .. ] }  }
///                           ^^^^^^^^ -------------- Contains offending access methods.
///
/// Algorithm: for each Shadowsocks access method, parse its cipher. If the parsing fails, discard
/// the access method in its entirety. Else keep it.
fn discard_invalid_shadowsocks_access_methods(settings: &mut Value) -> Option<()> {
    let custom_access_methods = settings
        .get_mut("api_access_methods")
        .and_then(|access_methods| access_methods.get_mut("custom"))
        .and_then(|custom_access_methods| custom_access_methods.as_array_mut())?;

    // `custom_access_methods` is a list of access method objects. An access method object looks like this:
    // { "access_method": { "custom": { "shadowsocks": { "cipher": <string> } } } }

    // If the access method is of type "shadowsocks", try to parse the "cipher" value. If the
    // parsing fails, discard the method.
    custom_access_methods.retain(|access_method| {
        if let Some(access_method) = access_method.get("access_method")
            && let Some(custom) = access_method.get("custom")
            && let Some(shadowsocks) = custom.get("shadowsocks")
            && let Some(cipher) = shadowsocks.get("cipher").and_then(Value::as_str)
        {
            ShadowsocksCipher::new(cipher).is_ok()
        } else {
            true
        }
    });
    Some(())
}

fn version_matches(settings: &Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V16 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    /// The migration should only discard invalid access methods - of course the valid ones should
    /// be saved.
    fn keep_valid_access_methods() {
        let mut old_settings = json!({
            "api_access_methods": {
              "custom": [
                {
                    "access_method": {
                        "custom": {
                            "shadowsocks": {
                                "cipher": "aes-256-gcm",
                                "endpoint": "[2001:ac8:40:22::bb01]:443",
                                "password": "mullvad"
                            }
                        }
                    }
                }
              ]
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        discard_invalid_shadowsocks_access_methods(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }

    #[test]
    /// Believe it or not, "finnish-tango" is not a valid Shadowsocks cipher.
    fn invalid_shadowsocks_access_method() {
        let mut old_settings = json!({
            "api_access_methods": {
              "custom": [
                {
                    "access_method": {
                        "custom": {
                            "shadowsocks": {
                                "cipher": "finnish-tango",
                                "endpoint": "[2001:ac8:40:22::bb01]:443",
                                "password": "mullvad"
                            }
                        }
                    }
                }
              ]
            }
        });
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
        discard_invalid_shadowsocks_access_methods(&mut old_settings).unwrap();
        insta::assert_snapshot!(serde_json::to_string_pretty(&old_settings).unwrap());
    }
}
