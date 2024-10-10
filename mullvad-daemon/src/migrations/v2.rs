#![allow(clippy::identity_op)]
use super::{Error, Result};
use mullvad_types::settings::SettingsVersion;
use std::time::Duration;

// ======================================================
// Section for vendoring types and values that
// this settings version depend on. See `mod.rs`.

pub const MIN_ROTATION_INTERVAL: Duration = Duration::from_secs(1 * 24 * 60 * 60);
pub const MAX_ROTATION_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

// ======================================================

pub fn migrate(settings: &mut serde_json::Value) -> Result<()> {
    if !version_matches(settings) {
        return Ok(());
    }

    log::info!("Migrating settings format to V3");

    // `show_beta_releases` used to be nullable
    if settings
        .get_mut("show_beta_releases")
        .map(|val| val.is_null())
        .unwrap_or(false)
    {
        settings
            .as_object_mut()
            .ok_or(Error::InvalidSettingsContent)?
            .remove("show_beta_releases");
    }

    let automatic_rotation = || -> Option<u64> {
        settings
            .get("tunnel_options")?
            .get("wireguard")?
            .get("automatic_rotation")
            .map(|ivl| ivl.as_u64())?
    }();

    if let Some(interval) = automatic_rotation {
        let new_ivl = match Duration::from_secs(60 * 60 * interval) {
            ivl if ivl < MIN_ROTATION_INTERVAL => {
                log::warn!("Increasing key rotation interval since it is below minimum");
                MIN_ROTATION_INTERVAL
            }
            ivl if ivl > MAX_ROTATION_INTERVAL => {
                log::warn!("Decreasing key rotation interval since it is above maximum");
                MAX_ROTATION_INTERVAL
            }
            ivl => ivl,
        };

        settings["tunnel_options"]["wireguard"]["rotation_interval"] = serde_json::json!(new_ivl);
        settings["tunnel_options"]["wireguard"]
            .as_object_mut()
            .ok_or(Error::InvalidSettingsContent)?
            .remove("automatic_rotation");
    }

    settings["settings_version"] = serde_json::json!(SettingsVersion::V3);

    Ok(())
}

fn version_matches(settings: &serde_json::Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == SettingsVersion::V2 as u64)
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use crate::migrations::load_seed;

    #[test]
    fn v2_migration() {
        let mut settings = load_seed("v2.json");
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings).unwrap());
    }
}
