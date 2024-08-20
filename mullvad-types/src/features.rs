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

impl IntoIterator for FeatureIndicators {
    type Item = FeatureIndicator;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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
    let server_ip_override = !settings.relay_overrides.is_empty(); // TODO: Should check if actually used

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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use talpid_types::net::{
        proxy::{ProxyEndpoint, ProxyType},
        Endpoint, ObfuscationEndpoint, TransportProtocol,
    };

    use super::*;

    #[test]
    fn test_one_indicator_at_a_time() {
        let mut settings = Settings::default();
        let mut endpoint = TunnelEndpoint {
            endpoint: Endpoint {
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                protocol: TransportProtocol::Udp,
            },
            tunnel_type: TunnelType::Wireguard,
            quantum_resistant: Default::default(),
            proxy: Default::default(),
            obfuscation: Default::default(),
            entry_endpoint: Default::default(),
            tunnel_interface: Default::default(),
            daita: Default::default(),
        };

        let mut expected_indicators: FeatureIndicators = [].into_iter().collect();

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators,
        "The default settings and TunnelEndpoint should not have any feature indicators. If this is not true anymore, please update this test.");

        settings.block_when_disconnected = true;
        expected_indicators.0.insert(FeatureIndicator::LockdownMode);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        settings
            .tunnel_options
            .dns_options
            .default_options
            .block_ads = true;

        expected_indicators
            .0
            .insert(FeatureIndicator::DnsContentBlockers);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        settings
            .tunnel_options
            .dns_options
            .default_options
            .block_ads = true;

        expected_indicators
            .0
            .insert(FeatureIndicator::DnsContentBlockers);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        settings.allow_lan = true;

        expected_indicators.0.insert(FeatureIndicator::LanSharing);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        // Setting mssfix without having an openVPN endpoint should not result in an indicator
        settings.tunnel_options.openvpn.mssfix = Some(1300);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        endpoint.tunnel_type = TunnelType::OpenVpn;
        expected_indicators.0.insert(FeatureIndicator::CustomMssFix);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        endpoint.proxy = Some(ProxyEndpoint {
            endpoint: Endpoint {
                address: SocketAddr::from(([1, 2, 3, 4], 443)),
                protocol: TransportProtocol::Tcp,
            },
            proxy_type: ProxyType::Shadowsocks,
        });

        expected_indicators.0.insert(FeatureIndicator::BridgeMode);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        endpoint.tunnel_type = TunnelType::Wireguard;
        expected_indicators
            .0
            .remove(&FeatureIndicator::CustomMssFix);
        expected_indicators.0.remove(&FeatureIndicator::BridgeMode);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        endpoint.quantum_resistant = true;
        expected_indicators
            .0
            .insert(FeatureIndicator::QuantumResistance);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        endpoint.entry_endpoint = Some(Endpoint {
            address: SocketAddr::from(([1, 2, 3, 4], 443)),
            protocol: TransportProtocol::Tcp,
        });
        expected_indicators.0.insert(FeatureIndicator::Multihop);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        endpoint.obfuscation = Some(ObfuscationEndpoint {
            endpoint: Endpoint {
                address: SocketAddr::from(([1, 2, 3, 4], 443)),
                protocol: TransportProtocol::Tcp,
            },
            obfuscation_type: ObfuscationType::Udp2Tcp,
        });
        expected_indicators.0.insert(FeatureIndicator::Udp2Tcp);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );
        endpoint.obfuscation.as_mut().unwrap().obfuscation_type = ObfuscationType::Shadowsocks;
        expected_indicators.0.remove(&FeatureIndicator::Udp2Tcp);
        expected_indicators.0.insert(FeatureIndicator::Shadowsocks);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        settings.tunnel_options.wireguard.mtu = Some(1300);
        expected_indicators.0.insert(FeatureIndicator::CustomMtu);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint),
            expected_indicators
        );

        #[cfg(daita)]
        {
            endpoint.daita = true;
            expected_indicators.0.insert(FeatureIndicator::Daita);
            assert_eq!(
                compute_feature_indicators(&settings, &endpoint),
                expected_indicators
            );
        }
    }
}
