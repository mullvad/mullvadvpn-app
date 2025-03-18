//! This module provides functionality for managing and updating the local relay list,
//! including support for loading these lists from disk & applying [overrides][`RelayOverride`].
//!
//! ## Overview
//!
//! The primary structure in this module, [`ParsedRelays`], holds information about the currently
//! available relays, including any overrides that have been applied to the original list fetched
//! from the Mullvad API or loaded from a local cache.

use std::{
    collections::HashMap,
    io::{self, BufReader},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use mullvad_types::{
    location::Location,
    relay_constraints::RelayOverride,
    relay_list::{Relay, RelayList},
};

use crate::{constants::UDP2TCP_PORTS, error::Error};

pub(crate) struct ParsedRelays {
    /// Tracks when the relay list was last updated.
    last_updated: SystemTime,
    /// The current list of relays, after applying [overrides][`RelayOverride`].
    parsed_list: RelayList,
    /// The original list of relays, as returned by the Mullvad relays API.
    original_list: RelayList,
    overrides: Vec<RelayOverride>,
}

impl ParsedRelays {
    /// Return a flat iterator with all relays
    pub fn relays(&self) -> impl Iterator<Item = &Relay> + Clone + '_ {
        self.parsed_list.relays()
    }

    /// Replace `self` with a new [`ParsedRelays`] based on [new_relays][`ParsedRelays`],
    /// bumping `self.last_updated` to the current system time.
    pub fn update(&mut self, new_relays: RelayList) {
        *self = Self::from_relay_list(new_relays, SystemTime::now(), &self.overrides);

        log::info!(
            "Updated relay inventory has {} relays",
            self.relays().count()
        );
    }

    /// Tracks when the relay list was last updated.
    ///
    /// The relay list can be updated by calling [`ParsedRelays::update`].
    pub const fn last_updated(&self) -> SystemTime {
        self.last_updated
    }

    pub fn etag(&self) -> Option<String> {
        self.parsed_list.etag.clone()
    }

    /// The original list of relays, as returned by the Mullvad relays API.
    pub const fn original_list(&self) -> &RelayList {
        &self.original_list
    }

    /// The current list of relays, after applying [overrides][`RelayOverride`].
    pub const fn parsed_list(&self) -> &RelayList {
        &self.parsed_list
    }

    /// Replace the previous set of [overrides][`RelayOverride`] with `new_overrides`.
    /// This will update `self.parsed_list` as a side-effect.
    pub(crate) fn set_overrides(&mut self, new_overrides: &[RelayOverride]) {
        self.parsed_list = Self::parse_relay_list(&self.original_list, new_overrides);
        self.overrides = new_overrides.to_vec();
    }

    pub(crate) fn empty() -> Self {
        ParsedRelays {
            last_updated: UNIX_EPOCH,
            parsed_list: RelayList::empty(),
            original_list: RelayList::empty(),
            overrides: vec![],
        }
    }

    /// Try to read the relays from disk, preferring the newer ones.
    pub(crate) fn from_file(
        cache_path: impl AsRef<Path>,
        resource_path: impl AsRef<Path>,
        overrides: &[RelayOverride],
    ) -> Result<Self, Error> {
        // prefer the resource path's relay list if the cached one doesn't exist or was modified
        // before the resource one was created.
        let cached_relays = Self::from_file_inner(cache_path, overrides);
        let bundled_relays = match Self::from_file_inner(resource_path, overrides) {
            Ok(bundled_relays) => bundled_relays,
            Err(e) => {
                log::error!("Failed to load bundled relays: {}", e);
                return cached_relays;
            }
        };

        if cached_relays
            .as_ref()
            .map(|cached| cached.last_updated > bundled_relays.last_updated)
            .unwrap_or(false)
        {
            cached_relays
        } else {
            Ok(bundled_relays)
        }
    }

    fn from_file_inner(path: impl AsRef<Path>, overrides: &[RelayOverride]) -> Result<Self, Error> {
        log::debug!("Reading relays from {}", path.as_ref().display());
        let (last_modified, file) =
            Self::open_file(path.as_ref()).map_err(Error::OpenRelayCache)?;
        let relay_list = serde_json::from_reader(BufReader::new(file)).map_err(Error::Serialize)?;

        Ok(Self::from_relay_list(relay_list, last_modified, overrides))
    }

    fn open_file(path: &Path) -> io::Result<(SystemTime, std::fs::File)> {
        let file = std::fs::File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((last_modified, file))
    }

    /// Create a new [`ParsedRelays`] from [relay_list][`RelayList`] and
    /// [overrides][`RelayOverride`]. This will apply `overrides` to `relay_list` and store the
    /// result in `self.parsed_list`.
    pub(crate) fn from_relay_list(
        relay_list: RelayList,
        last_updated: SystemTime,
        overrides: &[RelayOverride],
    ) -> Self {
        ParsedRelays {
            last_updated,
            parsed_list: Self::parse_relay_list(&relay_list, overrides),
            original_list: relay_list,
            overrides: overrides.to_vec(),
        }
    }

    /// Apply [overrides][`RelayOverride`] to [relay_list][`RelayList`], yielding an updated relay
    /// list.
    fn parse_relay_list(relay_list: &RelayList, overrides: &[RelayOverride]) -> RelayList {
        let mut remaining_overrides = HashMap::new();
        for relay_override in overrides {
            remaining_overrides.insert(relay_override.hostname.clone(), relay_override.to_owned());
        }

        let mut parsed_list = relay_list.clone();

        // Append data for obfuscation protocols ourselves, since the API does not provide it.
        if parsed_list.wireguard.udp2tcp_ports.is_empty() {
            parsed_list.wireguard.udp2tcp_ports.extend(UDP2TCP_PORTS);
        }

        // Add location and override relay data
        for country in &mut parsed_list.countries {
            for city in &mut country.cities {
                for relay in &mut city.relays {
                    // Append location data
                    relay.location = Location {
                        country: country.name.clone(),
                        country_code: country.code.clone(),
                        city: city.name.clone(),
                        city_code: city.code.clone(),
                        latitude: city.latitude,
                        longitude: city.longitude,
                    };

                    // Append overrides
                    if let Some(overrides) = remaining_overrides.remove(&relay.hostname) {
                        overrides.apply_to_relay(relay);
                    }
                }
            }
        }

        parsed_list
    }
}
