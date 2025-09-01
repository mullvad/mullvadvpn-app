//
//  ObfuscatorPortSelector.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

protocol RelayObfuscating {
    func obfuscate(
        _ relays: REST.ServerRelaysResponse,
        using tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> RelayObfuscation
}

struct RelayObfuscation {
    let allRelays: REST.ServerRelaysResponse
    let obfuscatedRelays: REST.ServerRelaysResponse
    let port: RelayConstraint<UInt16>
    var method: WireGuardObfuscationState
}

struct RelayObfuscator: RelayObfuscating {
    func obfuscate(
        _ relays: REST.ServerRelaysResponse,
        using tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> RelayObfuscation {
        var obfuscatedRelays = relays
        var port = tunnelSettings.relayConstraints.port

        let obfuscationMethod = ObfuscationMethodSelector.obfuscationMethodBy(
            connectionAttemptCount: connectionAttemptCount,
            tunnelSettings: tunnelSettings
        )

        return switch obfuscationMethod {
        case .udpOverTcp:
            UdpOverTcpObfuscator().obfuscate(
                relays,
                using: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            )
        case .shadowsocks:
            ShadowsocksObfuscator().obfuscate(
                relays,
                using: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            )
        case .quic:
            QuicObfuscator().obfuscate(
                relays,
                using: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            )
        default:
            RelayObfuscation(
                allRelays: relays,
                obfuscatedRelays: relays,
                port: port,
                method: obfuscationMethod
            )
        }
    }
}
