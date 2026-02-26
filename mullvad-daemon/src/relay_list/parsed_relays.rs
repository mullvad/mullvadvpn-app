//! This module provides functionality for reading and parsing [`CachedRelayList`]s from disk.

use std::io::{self, BufReader};
use std::path::Path;
use std::time::SystemTime;

use mullvad_api::CachedRelayList;

use crate::relay_list::RELAYS_FILENAME;
use crate::relay_list::error::Error;

/// Try to read the relays from disk, preferring the newer ones.
pub fn parse_relays_from_file(
    cache_dir: impl AsRef<Path>,
    resource_dir: impl AsRef<Path>,
) -> Result<CachedRelayList, Error> {
    let relay_list = match (
        from_file_inner(cache_dir.as_ref().join(RELAYS_FILENAME)),
        from_file_inner(resource_dir.as_ref().join(RELAYS_FILENAME)),
    ) {
        // prefer the resource path's relay list if the cached one doesn't exist or was modified
        // before the resource one was created.
        // If cache_time is later than install_time, return cached relay list
        (Ok((cached_relays, cache_time)), Ok((_, install_time))) if cache_time >= install_time => {
            cached_relays
        }
        // else, return the bundled relay list
        (Ok(_), Ok((bundled_relays, _))) => bundled_relays,
        (Ok((cached_relays, _)), _) => cached_relays,
        (_, Ok((bundled_relays, _))) => bundled_relays,
        (Err(cached_error), Err(bundled_error)) => {
            log::error!("Failed to load bundled relays: {bundled_error}");
            log::error!("Failed to load cached relays: {cached_error}");
            return Err(bundled_error);
        }
    };

    Ok(relay_list)
}

fn from_file_inner(path: impl AsRef<Path>) -> Result<(CachedRelayList, SystemTime), Error> {
    log::trace!("Reading relays from {}", path.as_ref().display());
    let (file, last_modified) = open_file(path).map_err(Error::OpenRelayCache)?;
    let cached_relay_list =
        serde_json::from_reader(BufReader::new(file)).map_err(Error::Serialize)?;

    Ok((cached_relay_list, last_modified))
}

fn open_file(path: impl AsRef<Path>) -> io::Result<(std::fs::File, SystemTime)> {
    let file = std::fs::File::open(path)?;
    let last_modified = file.metadata()?.modified()?;
    Ok((file, last_modified))
}
