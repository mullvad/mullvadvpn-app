//
//  TunnelConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 07/12/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

struct TunnelConfiguration {
    var isEnabled: Bool
    var localizedDescription: String
    var protocolConfiguration: NETunnelProviderProtocol
    var onDemandRules: [NEOnDemandRule]
    var isOnDemandEnabled: Bool

    init(excludeLocalNetworks: Bool, isOnDemandEnabled: Bool = true) {
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = ApplicationTarget.packetTunnel.bundleIdentifier
        protocolConfig.serverAddress = ""
        protocolConfig.includeAllNetworks = true
        protocolConfig.excludeLocalNetworks = excludeLocalNetworks

        let alwaysOnRule = NEOnDemandRuleConnect()
        alwaysOnRule.interfaceTypeMatch = .any

        isEnabled = true
        localizedDescription = "WireGuard"
        protocolConfiguration = protocolConfig
        onDemandRules = [alwaysOnRule]
        self.isOnDemandEnabled = isOnDemandEnabled
    }

    func apply(to manager: TunnelProviderManagerType) {
        manager.isEnabled = isEnabled
        manager.localizedDescription = localizedDescription
        manager.protocolConfiguration = protocolConfiguration
        manager.onDemandRules = onDemandRules
        manager.isOnDemandEnabled = isOnDemandEnabled
    }
}
