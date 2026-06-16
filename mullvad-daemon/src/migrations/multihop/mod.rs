//! The great multihop migration of 2026.
//!
//! In this migration the new Multihop tri-state is introduced (Auto, Never & Always) superceeding
//! the previous boolean value. It also deprecates the old "direct only" setting from DAITA, and
//! it's inverse ('automatic multihop') has been superceeded by the new Auto option for Multihop.

use super::Result;
use super::multihop::scenario::Scenario;
use super::multihop::update::migration;

use mullvad_types::settings::SettingsVersion;
use serde_json::{Value, json};

pub mod scenario;
pub mod settings;
mod update;

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
    use crate::migrations::multihop::settings::v17::SettingsBuilder;

    use super::*;

    /// Scenario 1A.
    /// # Expected outcome
    /// Multihop: When needed / "auto".
    #[test]
    fn scenario_1a() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(false)
            .filters(false)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::OneA);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 1B.
    /// # Expected outcome
    /// Multihop: Never.
    /// Filters: Copied to entry.
    #[test]
    fn scenario_1b() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(false)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::OneB);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 2.
    /// # Expected outcome
    /// Multihop: When needed / "auto".
    #[test]
    fn scenario_2() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(true)
            .direct_only(false)
            .filters(false)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::Two);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 3A.
    /// # Expected outcome
    /// Multihop: Never
    /// Filters: Copied to entry.
    #[test]
    fn scenario_3a() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(true)
            .direct_only(false)
            .magic_multihop(false)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::ThreeA);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 3B.
    /// # Expected outcome
    /// Multihop: Always
    /// Filters: Copied to entry.
    #[test]
    fn scenario_3b() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(true)
            .direct_only(false)
            .magic_multihop(true)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::ThreeB);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 4A.
    /// # Expected outcome
    /// Multihop: Never
    #[test]
    fn scenario_4a() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(true)
            .direct_only(true)
            .filters(false)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::FourA);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 4B.
    /// # Expected outcome
    /// Multihop: Never
    /// Filters: Copied to entry.
    #[test]
    fn scenario_4b() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(false)
            .daita(true)
            .direct_only(true)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::FourB);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 5a.
    /// # Expected outcome
    /// Multihop: Always
    #[test]
    fn scenario_5a() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(true)
            .daita(false)
            .filters(false)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::FiveA);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 5b.
    /// # Expected outcome
    /// Multihop: Always
    /// Filters: Copied to entry.
    #[test]
    fn scenario_5b() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(true)
            .daita(false)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::FiveB);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 6b.
    /// # Expected outcome
    /// Multihop: Always
    /// Filters: Copied to entry.
    #[test]
    fn scenario_6b() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(true)
            .daita(true)
            .direct_only(false)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::SixB);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 7a.
    /// # Expected outcome
    /// Multihop: Always
    #[test]
    fn scenario_7a() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(true)
            .daita(true)
            .direct_only(true)
            .filters(false)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::SevenA);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }

    /// Scenario 7b.
    /// # Expected outcome
    /// Multihop: Always
    /// Filters: Copied to entry.
    #[test]
    fn scenario_7b() -> anyhow::Result<()> {
        let settings = SettingsBuilder::new()
            .multihop(true)
            .daita(true)
            .direct_only(true)
            .filters(true)
            .build();
        let scenario = update::detect(&settings);
        assert_eq!(scenario, Scenario::SevenB);
        let mut settings = json!(settings);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        update::migrate(&mut settings, scenario);
        insta::assert_snapshot!(serde_json::to_string_pretty(&settings)?);
        Ok(())
    }
}
