use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use crate::settings::{DnsState, Settings};
use serde::{Deserialize, Serialize};
use talpid_types::net::{ObfuscationType, TunnelEndpoint, TunnelType};

/// Feature indicators are active settings that should be shown to the user to make them aware of
/// what is affecting their connection at any given time.
///
/// Note that the feature indicators are not ordered.
#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureIndicators(HashSet<FeatureIndicator>);

impl Debug for FeatureIndicators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut indicators: Vec<&str> = self.0.iter().map(|feature| feature.to_str()).collect();
        // Sort the features alphabetically (Just to have some order, arbitrarily chosen)
        indicators.sort();
        f.debug_tuple("FeatureIndicators")
            .field(&indicators)
            .finish()
    }
}

impl FeatureIndicators {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for FeatureIndicators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut indicators: Vec<&str> = self.0.iter().map(|feature| feature.to_str()).collect();
        // Sort the features alphabetically (Just to have some order, arbitrarily chosen)
        indicators.sort();

        write!(f, "{}", indicators.join(", "))
    }
}

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

    /// Whether DAITA (without multihop) is in use.
    /// Mutually exclusive with [FeatureIndicator::DaitaMultihop].
    Daita,

    /// Whether DAITA (with multihop) is in use.
    /// Mutually exclusive with [FeatureIndicator::Daita] and [FeatureIndicator::Multihop].
    DaitaMultihop,
}

impl FeatureIndicator {
    const fn to_str(&self) -> &'static str {
        match self {
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
            FeatureIndicator::DaitaMultihop => "DAITA: Multihop",
        }
    }
}

impl std::fmt::Display for FeatureIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let feature = self.to_str();
        write!(f, "{feature}")
    }
}

/// Calculate active [`FeatureIndicators`] from setting and endpoint information.
///
/// Note that [`FeatureIndicators`] are only applicable for the connected and connecting states, and
/// this function should not be called with arguments from different tunnel states.
///
/// Server ip override cannot be determined from the settings and endpoint, and has to be fetched
/// from the relay selector parameter generator.
pub fn compute_feature_indicators(
    settings: &Settings,
    endpoint: &TunnelEndpoint,
    server_ip_override: bool,
) -> FeatureIndicators {
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    let split_tunneling = settings.split_tunnel.enable_exclusions;
    #[cfg(not(any(windows, target_os = "android", target_os = "macos")))]
    let split_tunneling = false;

    #[cfg(not(target_os = "android"))]
    let lockdown_mode = settings.block_when_disconnected;
    let lan_sharing = settings.allow_lan;
    let dns_content_blockers = settings
        .tunnel_options
        .dns_options
        .default_options
        .any_blockers_enabled();
    let custom_dns = settings.tunnel_options.dns_options.state == DnsState::Custom;

    let generic_features = [
        (split_tunneling, FeatureIndicator::SplitTunneling),
        (lan_sharing, FeatureIndicator::LanSharing),
        (dns_content_blockers, FeatureIndicator::DnsContentBlockers),
        (custom_dns, FeatureIndicator::CustomDns),
        (server_ip_override, FeatureIndicator::ServerIpOverride),
        #[cfg(not(target_os = "android"))]
        (lockdown_mode, FeatureIndicator::LockdownMode),
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

            let mut daita_multihop = false;
            let mut multihop = false;

            if let crate::relay_constraints::RelaySettings::Normal(constraints) =
                &settings.relay_settings
            {
                multihop = endpoint.entry_endpoint.is_some()
                    && constraints.wireguard_constraints.use_multihop;

                #[cfg(daita)]
                {
                    // Detect whether we're using multihop, but it is not explicitly enabled.
                    daita_multihop = endpoint.daita
                        && endpoint.entry_endpoint.is_some()
                        && !constraints.wireguard_constraints.use_multihop
                }
            };

            // Daita is mutually exclusive with DaitaMultihop
            #[cfg(daita)]
            let daita = endpoint.daita && !daita_multihop;

            vec![
                (quantum_resistant, FeatureIndicator::QuantumResistance),
                (multihop, FeatureIndicator::Multihop),
                (udp_tcp, FeatureIndicator::Udp2Tcp),
                (shadowsocks, FeatureIndicator::Shadowsocks),
                (mtu, FeatureIndicator::CustomMtu),
                #[cfg(daita)]
                (daita, FeatureIndicator::Daita),
                (daita_multihop, FeatureIndicator::DaitaMultihop),
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

    use crate::relay_constraints::RelaySettings;

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
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators,
            "The default settings and TunnelEndpoint should not have any feature indicators. \
            If this is not true anymore, please update this test."
        );

        settings.block_when_disconnected = true;
        expected_indicators.0.insert(FeatureIndicator::LockdownMode);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
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
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        settings.allow_lan = true;

        expected_indicators.0.insert(FeatureIndicator::LanSharing);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        settings.tunnel_options.openvpn.mssfix = Some(1300);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators,
            "Setting mssfix without having an openVPN endpoint should not result in an indicator"
        );

        endpoint.tunnel_type = TunnelType::OpenVpn;
        expected_indicators.0.insert(FeatureIndicator::CustomMssFix);

        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
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
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        endpoint.tunnel_type = TunnelType::Wireguard;
        expected_indicators
            .0
            .remove(&FeatureIndicator::CustomMssFix);
        expected_indicators.0.remove(&FeatureIndicator::BridgeMode);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        endpoint.quantum_resistant = true;
        expected_indicators
            .0
            .insert(FeatureIndicator::QuantumResistance);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        endpoint.entry_endpoint = Some(Endpoint {
            address: SocketAddr::from(([1, 2, 3, 4], 443)),
            protocol: TransportProtocol::Tcp,
        });
        if let RelaySettings::Normal(constraints) = &mut settings.relay_settings {
            constraints.wireguard_constraints.use_multihop = true;
        };
        expected_indicators.0.insert(FeatureIndicator::Multihop);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
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
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );
        endpoint.obfuscation.as_mut().unwrap().obfuscation_type = ObfuscationType::Shadowsocks;
        expected_indicators.0.remove(&FeatureIndicator::Udp2Tcp);
        expected_indicators.0.insert(FeatureIndicator::Shadowsocks);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        settings.tunnel_options.wireguard.mtu = Some(1300);
        expected_indicators.0.insert(FeatureIndicator::CustomMtu);
        assert_eq!(
            compute_feature_indicators(&settings, &endpoint, false),
            expected_indicators
        );

        #[cfg(daita)]
        {
            endpoint.daita = true;
            expected_indicators.0.insert(FeatureIndicator::Daita);
            assert_eq!(
                compute_feature_indicators(&settings, &endpoint, false),
                expected_indicators
            );

            // Should not change regardless of whether `use_multihop_if_necessary` is true, since
            // multihop is enabled explicitly
            settings
                .tunnel_options
                .wireguard
                .daita
                .use_multihop_if_necessary = false;
            assert_eq!(
                compute_feature_indicators(&settings, &endpoint, false),
                expected_indicators,
            );

            // Here we mock that multihop was automatically enabled by DAITA.
            // We enable `use_multihop_if_necessary` again and disable the multihop setting, while
            // keeping the entry relay. In this scenario, we should still get a Multihop
            // indicator.
            settings
                .tunnel_options
                .wireguard
                .daita
                .use_multihop_if_necessary = true;
            if let RelaySettings::Normal(constraints) = &mut settings.relay_settings {
                constraints.wireguard_constraints.use_multihop = false;
            };
            expected_indicators
                .0
                .insert(FeatureIndicator::DaitaMultihop);
            expected_indicators.0.remove(&FeatureIndicator::Daita);
            expected_indicators.0.remove(&FeatureIndicator::Multihop);
            assert_eq!(
                compute_feature_indicators(&settings, &endpoint, false),
                expected_indicators,
                "DaitaDirectOnly should be enabled"
            );

            // If we also remove the entry relay, we should not get a multihop indicator
            expected_indicators.0.insert(FeatureIndicator::Daita);
            endpoint.entry_endpoint = None;
            expected_indicators.0.remove(&FeatureIndicator::Multihop);
            expected_indicators
                .0
                .remove(&FeatureIndicator::DaitaMultihop);
            assert_eq!(
                compute_feature_indicators(&settings, &endpoint, false),
                expected_indicators,
                "DaitaDirectOnly should be enabled"
            );
        }

        // NOTE: If this match statement fails to compile, it means that a new feature indicator has
        // been added. Please update this test to include the new feature indicator.
        match FeatureIndicator::QuantumResistance {
            FeatureIndicator::QuantumResistance => {}
            FeatureIndicator::Multihop => {}
            FeatureIndicator::BridgeMode => {}
            FeatureIndicator::SplitTunneling => {}
            FeatureIndicator::LockdownMode => {}
            FeatureIndicator::Udp2Tcp => {}
            FeatureIndicator::Shadowsocks => {}
            FeatureIndicator::LanSharing => {}
            FeatureIndicator::DnsContentBlockers => {}
            FeatureIndicator::CustomDns => {}
            FeatureIndicator::ServerIpOverride => {}
            FeatureIndicator::CustomMtu => {}
            FeatureIndicator::CustomMssFix => {}
            FeatureIndicator::Daita => {}
            FeatureIndicator::DaitaMultihop => {}
        }
    }
}
