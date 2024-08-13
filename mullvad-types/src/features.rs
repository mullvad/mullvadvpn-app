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
    DaitaUseAnywhere,
}

impl std::fmt::Display for FeatureIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let feature = match self {
            FeatureIndicator::QuantumResistance => "Quantum Resistance",
            FeatureIndicator::Multihop => "Multihop",
            FeatureIndicator::BridgeMode => "Bridge Mode",
            FeatureIndicator::SplitTunneling => "Split Tunneling",
            FeatureIndicator::LockdownMode => "Lockdown Mode",
            FeatureIndicator::Udp2Tcp => "Udp2Tcp",
            FeatureIndicator::LanSharing => "LAN Sharing",
            FeatureIndicator::DnsContentBlockers => "Dns Content Blocker",
            FeatureIndicator::CustomDns => "Custom Dns",
            FeatureIndicator::ServerIpOverride => "Server Ip Override",
            FeatureIndicator::CustomMtu => "Custom MTU",
            FeatureIndicator::CustomMssFix => "Custom MSS",
            FeatureIndicator::Daita => "DAITA",
            FeatureIndicator::DaitaUseAnywhere => "Use Anywhere (DAITA)",
        };
        write!(f, "{feature}")
    }
}
