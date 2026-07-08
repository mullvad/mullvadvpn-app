pub mod error;
pub mod parsed_relays;
pub mod update;

/// Where the relay list is cached on disk.
const RELAYS_FILENAME: &str = "relays.json";
