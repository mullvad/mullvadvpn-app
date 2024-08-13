use std::collections::HashSet;

use crate::settings::{DnsState, Settings};
use serde::{Deserialize, Serialize};
use talpid_types::net::{ObfuscationType, TunnelEndpoint, TunnelType};

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
    Shadowsocks,
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
            FeatureIndicator::Shadowsocks => "Shadowsocks",
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

/// Calculate active [`FeatureIndicators`] from setting and endpoint information.
///
/// Note that [`FeatureIndicators`] are only applicable for the connected and connecting states, and
/// this function should not be called with arguments from different tunnel states.
pub fn compute_feature_indicators(
    settings: &Settings,
    endpoint: &TunnelEndpoint,
) -> FeatureIndicators {
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    let split_tunneling = settings.split_tunnel.enable_exclusions;
    #[cfg(not(any(windows, target_os = "android", target_os = "macos")))]
    let split_tunneling = false;

    let lockdown_mode = settings.block_when_disconnected;
    let lan_sharing = settings.allow_lan;
    let dns_content_blockers = settings
        .tunnel_options
        .dns_options
        .default_options
        .any_blockers_enabled();
    let custom_dns = settings.tunnel_options.dns_options.state == DnsState::Custom;
    let server_ip_override = !settings.relay_overrides.is_empty();

    let generic_features = [
        (split_tunneling, FeatureIndicator::SplitTunneling),
        (lockdown_mode, FeatureIndicator::LockdownMode),
        (lan_sharing, FeatureIndicator::LanSharing),
        (dns_content_blockers, FeatureIndicator::DnsContentBlockers),
        (custom_dns, FeatureIndicator::CustomDns),
        (server_ip_override, FeatureIndicator::ServerIpOverride),
    ];

    // Pick protocol-specific features and whether they are currently enabled.
    let protocol_features = match endpoint.tunnel_type {
        TunnelType::OpenVpn => {
            let bridge_mode = endpoint.proxy.is_some();
            let mss_fix = settings.tunnel_options.openvpn.mssfix.is_some();

            vec![
                (bridge_mode, FeatureIndicator::BridgeMode),
                (mss_fix, FeatureIndicator::CustomMssFix),
            ]
        }
        TunnelType::Wireguard => {
            let quantum_resistant = endpoint.quantum_resistant;
            let multihop = endpoint.entry_endpoint.is_some();
            let udp_tcp = endpoint
                .obfuscation
                .as_ref()
                .filter(|obfuscation| obfuscation.obfuscation_type == ObfuscationType::Udp2Tcp)
                .is_some();
            let shadowsocks = endpoint
                .obfuscation
                .as_ref()
                .filter(|obfuscation| obfuscation.obfuscation_type == ObfuscationType::Shadowsocks)
                .is_some();

            let mtu = settings.tunnel_options.wireguard.mtu.is_some();

            #[cfg(daita)]
            let daita = endpoint.daita;

            vec![
                (quantum_resistant, FeatureIndicator::QuantumResistance),
                (multihop, FeatureIndicator::Multihop),
                (udp_tcp, FeatureIndicator::Udp2Tcp),
                (shadowsocks, FeatureIndicator::Shadowsocks),
                (mtu, FeatureIndicator::CustomMtu),
                #[cfg(daita)]
                (daita, FeatureIndicator::Daita),
            ]
        }
    };

    // use the booleans to filter into a list of only the active features
    generic_features
        .into_iter()
        .chain(protocol_features)
        .filter_map(|(active, feature)| active.then_some(feature))
        .collect()
}
