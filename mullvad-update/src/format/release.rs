//! App release

use serde::{Deserialize, Serialize};

use super::installer::Installer;
use crate::version::{Rollout, is_complete_rollout};

/// App release
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Release {
    /// Mullvad app version
    pub version: mullvad_version::Version,
    /// Changelog entries
    pub changelog: String,
    /// Installer details for different architectures
    pub installers: Vec<Installer>,
    /// Fraction of users that should receive the new version
    #[serde(default = "Rollout::complete")]
    #[serde(skip_serializing_if = "is_complete_rollout")]
    pub rollout: Rollout,
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
