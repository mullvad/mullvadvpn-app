//! Given a top-level settings blob, update the subset of the settings blob represented by [`v17::__Settings`].

use crate::migrations::Error;
use crate::migrations::multihop::scenario::Scenario;
use crate::migrations::multihop::settings::v18::__Entry;
use crate::migrations::multihop::settings::{v17, v18};

use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

/// Perform the migration on a settings blob.
pub(crate) fn run(
    settings: &mut Value,
    cache_dir: impl AsRef<Path>,
    resource_dir: impl AsRef<Path>,
) -> Result<Scenario, Error> {
    // Parse the current settings blob to a structured format.
    let input = v17::__Settings::deserialize(settings.clone())
        .map_err(Error::Deserialize)?
        .check_magic_mulithop(cache_dir, resource_dir)?;
    // Detect which scenario the migration led to.
    let scenario = detect(&input);
    // Run the actual migration
    migrate(settings, scenario.clone());
    Ok(scenario)
}

pub(crate) fn migrate(settings: &mut Value, scenario: Scenario) {
    use v18::__Entry::*;
    use v18::__Multihop::*;
    match scenario {
        Scenario::OneA => relay_settings_migration(settings, WhenNeeded, false, Automatic(false)),
        Scenario::OneB => {
            relay_settings_migration(settings, Never, true, Automatic(false));
        }
        Scenario::Two => relay_settings_migration(settings, WhenNeeded, false, Automatic(false)),
        Scenario::ThreeA => {
            relay_settings_migration(settings, Never, true, Automatic(false));
        }
        Scenario::ThreeB {
            last_known_working_location,
        } => {
            relay_settings_migration(
                settings,
                Always,
                true,
                LastKnownWorking(last_known_working_location),
            );
        }
        Scenario::FourA => relay_settings_migration(settings, Never, false, Automatic(false)),
        Scenario::FourB => {
            relay_settings_migration(settings, Never, true, Automatic(false));
        }
        Scenario::FiveA => relay_settings_migration(settings, Always, false, Automatic(false)),
        Scenario::FiveB => {
            relay_settings_migration(settings, Always, true, Automatic(false));
        }
        Scenario::SixA => {
            relay_settings_migration(settings, Always, false, Automatic(true));
        }
        Scenario::SixB => {
            relay_settings_migration(settings, Always, true, Automatic(false));
        }
        Scenario::SevenA => {
            relay_settings_migration(settings, Always, false, Automatic(false));
        }
        Scenario::SevenB => {
            relay_settings_migration(settings, Always, true, Automatic(false));
        }
    };
    // Update DAITA settings value. Notably `use_multihop_if_necessary` is gone.
    daita_migration(settings);
    recents_migration(settings);
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
        (false, true, false, true, true) => ThreeB {
            last_known_working_location: settings.magic_multihop.as_ref().unwrap().clone(),
        },
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
    automatic_entry: __Entry,
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

fn recents_migration(settings: &mut Value) {
    let Some(recents_raw) = settings.get_mut("recents") else {
        // `recents` may be absent in older settings files. Nothing to migrate.
        return;
    };
    let recents: Option<Vec<v17::__Recent>> = serde_json::from_value(recents_raw.clone())
        .expect("It should be safe to cast recents to v17::__Recent");
    let recents_v18: Option<Vec<v18::__Recent>> =
        recents.map(|r| r.into_iter().map(|r| r.into()).collect());
    *recents_raw = json!(recents_v18);
}

#[cfg(test)]
mod test {
    use super::recents_migration;
    use serde_json::json;

    #[test]
    fn recents_migration_wraps_multihop_entry_in_only() {
        let mut settings = json!({
            "recents": [
                {"Multihop": {
                    "entry": {"location": {"country": "se"}},
                    "exit": {"custom_list": {"list_id": "df612270-79a4-47e9-92e7-3405c92f7678"}}
                }},
                {"Multihop": {
                    "entry": {"custom_list": {"list_id": "abc"}},
                    "exit": {"location": {"country": "fi"}}
                }},
                {"Singlehop": {"location": {"hostname": ["be", "bru", "be-bru-wg-103"]}}}
            ]
        });
        recents_migration(&mut settings);
        assert_eq!(
            settings,
            json!({
                "recents": [
                    {"Multihop": {
                        "entry": {"only": {"location": {"country": "se"}}},
                        "exit": {"custom_list": {"list_id": "df612270-79a4-47e9-92e7-3405c92f7678"}}
                    }},
                    {"Multihop": {
                        "entry": {"only": {"custom_list": {"list_id": "abc"}}},
                        "exit": {"location": {"country": "fi"}}
                    }},
                    {"Singlehop": {"location": {"hostname": ["be", "bru", "be-bru-wg-103"]}}}
                ]
            })
        );
    }
}
