//
//  TunnelSettingsV2.swift
//  MullvadVPN
//
//  Created by pronebird on 27/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

public struct TunnelSettingsV2: Codable, Equatable, TunnelSettings {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        TunnelSettingsV3(
            relayConstraints: relayConstraints,
            dnsSettings: dnsSettings,
            wireGuardObfuscation: WireGuardObfuscationSettings()
        )
    }
}
