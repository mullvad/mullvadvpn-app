//! App release

use serde::{Deserialize, Serialize};

/// App release
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Release {
    /// Mullvad app version
    pub version: mullvad_version::Version,
}

impl PartialEq for Release {
    fn eq(&self, other: &Self) -> bool {
        self.version.eq(&other.version)
    }
}

impl PartialOrd for Release {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.version.partial_cmp(&other.version)
    }
}
