// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

public struct TunnelSettingsV3: Codable, Equatable, TunnelSettings {
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
        TunnelSettingsV4(
            relayConstraints: relayConstraints,
            dnsSettings: dnsSettings,
            wireGuardObfuscation: wireGuardObfuscation,
            tunnelQuantumResistance: .on
        )
    }

    public var debugDescription: String {
        "TunnelSettingsV3(relayConstraints: \(relayConstraints), dnsSettings: \(dnsSettings), wireGuardObfuscation: \(wireGuardObfuscation))"
    }
}
