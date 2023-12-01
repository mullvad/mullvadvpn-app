//
//  TunnelSettingsV4.swift
//  MullvadSettings
//
//  Created by Jon Petersson on 2023-12-01.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

public struct TunnelSettingsV4: Codable, Equatable, TunnelSettings {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    /// WireGuard obfuscation settings
    public var wireGuardObfuscation: WireGuardObfuscationSettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }
}
