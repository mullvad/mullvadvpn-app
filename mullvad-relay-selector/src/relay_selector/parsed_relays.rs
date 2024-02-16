//! TODO(markus): Document this!

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
    last_updated: SystemTime,
    parsed_list: RelayList,
    original_list: RelayList,
    overrides: Vec<RelayOverride>,
}

impl ParsedRelays {
    /// Return a flat iterator with all relays
    pub fn relays(&self) -> impl Iterator<Item = &Relay> + Clone + '_ {
        self.parsed_list.relays()
    }

    pub fn update(&mut self, new_relays: RelayList) {
        *self = Self::from_relay_list(new_relays, SystemTime::now(), &self.overrides);

        log::info!(
            "Updated relay inventory has {} relays",
            self.relays().count()
        );
    }

    pub const fn last_updated(&self) -> SystemTime {
        self.last_updated
    }

    pub fn etag(&self) -> Option<String> {
        self.parsed_list.etag.clone()
    }

    pub const fn original_list(&self) -> &RelayList {
        &self.original_list
    }

    pub const fn parsed_list(&self) -> &RelayList {
        &self.parsed_list
    }

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

    fn parse_relay_list(relay_list: &RelayList, overrides: &[RelayOverride]) -> RelayList {
        let mut remaining_overrides = HashMap::new();
        for relay_override in overrides {
            remaining_overrides.insert(
                relay_override.hostname.to_owned(),
                relay_override.to_owned(),
            );
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
                    relay.location = Some(Location {
                        country: country.name.clone(),
                        country_code: country.code.clone(),
                        city: city.name.clone(),
                        city_code: city.code.clone(),
                        latitude: city.latitude,
                        longitude: city.longitude,
                    });

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
