//! Given a top-level settings blob, update the subset of the settings blob represented by [`v17::__Settings`].

use crate::migrations::Error;
use crate::migrations::multihop::scenario::Scenario;
use crate::migrations::multihop::settings::{v17, v18};

use serde::Deserialize;
use serde_json::{Value, json};

/// Perform the migration on a settings blob.
pub(crate) fn run(
    settings: &mut Value,
    cache_dir: &Path,
    resource_dir: &Path,
) -> Result<Scenario, Error> {
    // Parse the current settings blob to a structured format.
    let input = v17::__Settings::deserialize(settings.clone())
        .map_err(Error::Deserialize)?
        .check_magic_mulithop(cache_dir, resource_dir)?;
    // Detect which scenario the migration led to.
    let scenario = detect(&input);
    // Run the actual migration
    migrate(settings, scenario);
    Ok(scenario)
}

pub(crate) fn migrate(settings: &mut Value, scenario: Scenario) {
    use v18::__Multihop::*;
    match scenario {
        Scenario::OneA => relay_settings_migration(settings, WhenNeeded, false, false),
        Scenario::OneB => {
            relay_settings_migration(settings, Never, true, false);
        }
        Scenario::Two => relay_settings_migration(settings, WhenNeeded, false, false),
        Scenario::ThreeA => {
            relay_settings_migration(settings, Never, true, false);
        }
        Scenario::ThreeB => {
            relay_settings_migration(settings, Always, true, false);
        }
        Scenario::FourA => relay_settings_migration(settings, Never, false, false),
        Scenario::FourB => {
            relay_settings_migration(settings, Never, true, false);
        }
        Scenario::FiveA => relay_settings_migration(settings, Always, false, false),
        Scenario::FiveB => {
            relay_settings_migration(settings, Always, true, false);
        }
        Scenario::SixA => {
            relay_settings_migration(settings, Always, false, true);
        }
        Scenario::SixB => {
            relay_settings_migration(settings, Always, true, false);
        }
        Scenario::SevenA => {
            relay_settings_migration(settings, Always, false, false);
        }
        Scenario::SevenB => {
            relay_settings_migration(settings, Always, true, false);
        }
    };
    // Update DAITA settings value. Notably `use_multihop_if_necessary` is gone.
    daita_migration(settings);
}

/// Detect which scenario we are dealing with based on the previous settings.
///
/// See [crate::migrations::multihop::scenario].
pub(crate) fn detect(settings: &v17::__Settings) -> Scenario {
    use Scenario::*;
    let (multihop, daita, direct_only, magic_multihop, filters) = (
        settings.legacy_multihop(),
        settings.daita(),
        settings.direct_only(),
        settings.magic_multihop(),
        settings.filters(),
    );
    match (multihop, daita, direct_only, magic_multihop, filters) {
        (false, false, _, _, false) => OneA,
        (false, false, _, _, true) => OneB,
        (false, true, false, _, false) => Two,
        (false, true, false, false, true) => ThreeA,
        (false, true, false, true, true) => ThreeB,
        (false, true, true, _, false) => FourA,
        (false, true, true, _, true) => FourB,
        (true, false, _, _, false) => FiveA,
        (true, false, _, _, true) => FiveB,
        (true, true, false, _, false) => SixA,
        (true, true, false, _, true) => SixB,
        (true, true, true, _, false) => SevenA,
        (true, true, true, _, true) => SevenB,
    }
}

/// Update the multihop and filters of an existing settings blob to the new [`v18`] format.
///
/// See [`v18::__RelaySettings::migrate`] for details.
fn relay_settings_migration(
    settings: &mut Value,
    value: v18::__Multihop,
    filters: bool,
    automatic_entry: bool,
) {
    let Some(relay_settings) = v17::__Settings::relay_settings(settings) else {
        log::debug!("Did not update relay settings to {value:?}");
        if filters {
            log::debug!("Filter migration did not run either");
        }
        return;
    };
    let new = v18::__RelaySettings::migrate(
        serde_json::from_value(relay_settings.clone())
            .expect("It should be safe to cast settings to v17::__RelaySettings"),
        value,
        filters,
        automatic_entry,
    );
    *relay_settings = json!(new);
}

/// Update the daita value of an existing settings blob to the new [`v18::__WireguardSettings`] format.
///
/// See [`v18::__WireguardSettings::migrate`] for details.
fn daita_migration(settings: &mut Value) {
    let wg_settings = v17::__Settings::wireguard_settings(settings);
    let wg: v17::__WireguardSettings = serde_json::from_value(wg_settings.clone())
        .expect("It should be safe to cast settings to v17::__WireguardSettings");
    *wg_settings = json!(v18::__WireguardSettings::migrate(wg));
}
