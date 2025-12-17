//! This module provides functionality for parsing the local relay list,
//! including support for loading these lists from disk & applying [overrides][`RelayOverride`].

use std::collections::HashMap;
use std::io::{self, BufReader};
use std::path::Path;
use std::time::SystemTime;

use mullvad_api::{CachedRelayList, ETag};
use mullvad_relay_selector::UDP2TCP_PORTS;
use mullvad_types::location::Location;
use mullvad_types::relay_constraints::RelayOverride;
use mullvad_types::relay_list::{BridgeList, RelayList};

use crate::relay_list::RELAYS_FILENAME;
use crate::relay_list::error::Error;

// TODO: Remove ?
pub struct ParseRelays;

impl ParseRelays {
    /// Try to read the relays from disk, preferring the newer ones.
    pub fn from_file(
        cache_dir: impl AsRef<Path>,
        resource_dir: impl AsRef<Path>,
        overrides: Vec<RelayOverride>,
    ) -> Result<(RelayList, BridgeList, Option<ETag>), Error> {
        let relay_list = match (
            Self::from_file_inner(cache_dir.as_ref().join(RELAYS_FILENAME)),
            Self::from_file_inner(resource_dir.as_ref().join(RELAYS_FILENAME)),
        ) {
            // prefer the resource path's relay list if the cached one doesn't exist or was modified
            // before the resource one was created.
            // If cache_time is later than install_time, return cached relay list
            (Ok((cached_relays, cache_time)), Ok((_, install_time)))
                if cache_time >= install_time =>
            {
                cached_relays
            }
            // else, return the bundled relay list
            (Ok(_), Ok((bundled_relays, _))) => bundled_relays,
            (Ok((cached_relays, _)), _) => cached_relays,
            (_, Ok((bundled_relays, _))) => bundled_relays,
            (Err(_cached_error), Err(bundled_error)) => return Err(bundled_error),
        };

        let etag = relay_list.etag().cloned();
        let (relay_list, bridge_list) = relay_list.into_internal_repr();
        // Apply overrides at this point
        let parsed_list = Self::apply_overrides(relay_list.clone(), overrides.clone());
        Ok((parsed_list, bridge_list, etag))
    }

    fn from_file_inner(path: impl AsRef<Path>) -> Result<(CachedRelayList, SystemTime), Error> {
        log::debug!("Reading relays from {}", path.as_ref().display());
        let (file, last_modified) =
            Self::open_file(path.as_ref()).map_err(Error::OpenRelayCache)?;
        let cached_relay_list =
            serde_json::from_reader(BufReader::new(file)).map_err(Error::Serialize)?;

        Ok((cached_relay_list, last_modified))
    }

    fn open_file(path: &Path) -> io::Result<(std::fs::File, SystemTime)> {
        let file = std::fs::File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((file, last_modified))
    }

    /// Apply [overrides][`RelayOverride`] to [relay_list][`RelayList`], yielding an updated relay
    /// list.
    fn apply_overrides(mut parsed_list: RelayList, overrides: Vec<RelayOverride>) -> RelayList {
        let mut remaining_overrides = HashMap::new();
        for relay_override in overrides {
            remaining_overrides.insert(relay_override.hostname.clone(), relay_override);
        }

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
