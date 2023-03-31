//
//  TunnelSettingsV2+VPN.swift
//  MullvadVPN
//
//  Created by pronebird on 31/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

extension TunnelSettingsV2 {
    func makeTunnelConfiguration() -> TunnelConfiguration {
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = ApplicationConfiguration.packetTunnelExtensionIdentifier
        protocolConfig.serverAddress = ""

        return TunnelConfiguration(
            isEnabled: true,
            localizedDescription: "WireGuard",
            protocolConfiguration: protocolConfig,
            onDemandRules: makeOnDemandRules(),
            isOnDemandEnabled: true
        )
    }

    func makeOnDemandRules() -> [NEOnDemandRule] {
        var onDemandRules: [NEOnDemandRule] = []

        // Disconnect VPN when on trusted Wi-Fi network.
        if trustedNetworkSettings.effectiveIsEnabled {
            let disconnectRule = NEOnDemandRuleDisconnect()
            disconnectRule.ssidMatch = trustedNetworkSettings.ssids
            disconnectRule.interfaceTypeMatch = .wiFi
            onDemandRules.append(disconnectRule)
        }

        // Connect VPN in other circumstances.
        let connectRule = NEOnDemandRuleConnect()
        connectRule.interfaceTypeMatch = .any
        onDemandRules.append(connectRule)

        return onDemandRules
    }
}
