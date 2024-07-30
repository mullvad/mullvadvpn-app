use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Feature indicators are active settings that should be shown to the user to make them aware of
/// what is affecting their connection at any given time.
///
/// Note that the feature indicators are not ordered.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureIndicators(HashSet<FeatureIndicator>);

impl FeatureIndicators {
    pub fn active_features(&self) -> impl Iterator<Item = FeatureIndicator> {
        self.0.clone().into_iter()
    }
}

impl FromIterator<FeatureIndicator> for FeatureIndicators {
    fn from_iter<T: IntoIterator<Item = FeatureIndicator>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// All possible feature indicators. These represent a subset of all VPN settings in a
/// non-technical fashion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureIndicator {
    QuantumResistance,
    Multihop,
    BridgeMode,
    SplitTunneling,
    LockdownMode,
    Udp2Tcp,
    LanSharing,
    DnsContentBlockers,
    CustomDns,
    ServerIpOverride,
    CustomMtu,
    CustomMssFix,
    Daita,
}
