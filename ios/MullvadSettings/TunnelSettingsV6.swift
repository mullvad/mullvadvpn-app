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

public struct TunnelSettingsV6: Codable, Equatable, TunnelSettings, Sendable {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    /// WireGuard obfuscation settings
    public var wireGuardObfuscation: WireGuardObfuscationSettings

    /// Whether Post Quantum exchanges are enabled.
    public var tunnelQuantumResistance: TunnelQuantumResistance

    /// Whether Multihop is enabled.
    public var tunnelMultihopState: MultihopStateV1

    /// DAITA settings.
    public var daita: DAITASettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        tunnelQuantumResistance: TunnelQuantumResistance = .on,
        tunnelMultihopState: MultihopStateV1 = .off,
        daita: DAITASettings = DAITASettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.tunnelQuantumResistance = tunnelQuantumResistance
        self.tunnelMultihopState = tunnelMultihopState
        self.daita = daita
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        TunnelSettingsV7(
            relayConstraints: relayConstraints,
            dnsSettings: dnsSettings,
            wireGuardObfuscation: wireGuardObfuscation,
            tunnelQuantumResistance: tunnelQuantumResistance,
            tunnelMultihopState: tunnelMultihopState,
            daita: daita,
            includeAllNetworks: IncludeAllNetworksSettings()
        )
    }

    public var debugDescription: String {
        "TunnelSettingsV6(relayConstraints: \(relayConstraints), dnsSettings: \(dnsSettings), wireGuardObfuscation: \(wireGuardObfuscation), tunnelQuantumResistance: \(tunnelQuantumResistance), tunnelMultihopState: \(tunnelMultihopState), daita: \(daita))"
    }
}
