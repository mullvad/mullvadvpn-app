//! Payload from the Mullvad metadata API.

use serde::{Deserialize, Serialize};

use super::release::Release;

/// Android releases
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct AndroidReleases {
    /// Available app releases
    pub releases: Vec<Release>,
}
