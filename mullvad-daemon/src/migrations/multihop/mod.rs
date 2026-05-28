//! The great multihop migration of 2026.
//!
//! In this migration the new Multihop tri-state is introduced (Auto, Never & Always) superceeding
//! the previous boolean value. It also deprecates the old "direct only" setting from DAITA, and
//! it's inverse ('automatic multihop') has been superceeded by the new Auto option for Multihop.
#![allow(unused)] // TODO: Remove before merging to main.

mod migration;
pub mod scenario;
pub mod settings;

use super::Result;
use super::multihop::scenario::Scenario;

use mullvad_types::settings::SettingsVersion;
use serde_json::{Value, json};

const SETTING: SettingsVersion = SettingsVersion::V18;
const PREVIOUS_SETTING: SettingsVersion = SettingsVersion::V17;

/// NOTE: This migration has been closed.
///
/// If `Ok(none)` is returned, the migration has already run.
pub fn migrate(settings: &mut Value) -> Result<Option<Scenario>> {
    if !version_matches(settings) {
        return Ok(None);
    }
    log::info!("Running Multihop/filter migration settings format to ");
    // Perform the migration
    let scenario = migration(settings)?;
    // Done. propagate the scenario.
    log::info!("Successfully ran Multihop/filter migration. Enjoy!");
    settings["settings_version"] = json!(SETTING);
    Ok(Some(scenario))
}

pub(crate) fn version_matches(settings: &Value) -> bool {
    settings
        .get("settings_version")
        .map(|version| version == PREVIOUS_SETTING as u64)
        .unwrap_or(false)
}

#[cfg(test)]
/// The setup for each scenario is broken down in [scenario].
mod test {
    use super::*;

    /// Scenario 1A.
    /// # Expected outcome
    /// Multihop: When needed.
    fn scenario_1a() -> Result<()> {
        Ok(())
    }
}
