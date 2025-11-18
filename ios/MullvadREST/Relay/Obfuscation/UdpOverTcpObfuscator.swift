//
//  UdpOverTcpObfuscator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-09-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct UdpOverTcpObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

    func obfuscate() -> RelayObfuscation {
        RelayObfuscation(
            allRelays: relays,
            obfuscatedRelays: relays,
            port: obfuscateUdpOverTcpPort(
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ),
            method: .udpOverTcp
        )
    }

    private func obfuscateUdpOverTcpPort(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) -> RelayConstraint<UInt16> {
        switch tunnelSettings.wireGuardObfuscation.udpOverTcpPort {
        case .automatic:
            return [.only(80), .only(443)].randomElement()!
        case .port5001:
            return .only(5001)
        case .port80:
            return .only(80)
        case .port443:
            return .only(443)
        }
    }
}
