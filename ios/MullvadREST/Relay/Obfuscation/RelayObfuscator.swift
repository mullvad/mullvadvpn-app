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
    var relays: REST.ServerRelaysResponse { get }
    var tunnelSettings: LatestTunnelSettings { get }
    var connectionAttemptCount: UInt { get }
    func obfuscate() throws -> RelayObfuscation
}

struct RelayObfuscation {
    let allRelays: REST.ServerRelaysResponse
    let obfuscatedRelays: REST.ServerRelaysResponse
    let port: RelayConstraint<UInt16>
    var method: WireGuardObfuscationState
}

struct RelayObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt
    let bias: any ObfuscationProviding

    func obfuscate() throws -> RelayObfuscation {
        let obfuscationMethod = ObfuscationMethodSelector.obfuscationMethodBy(
            connectionAttemptCount: connectionAttemptCount,
            tunnelSettings: tunnelSettings,
            obfuscationBypass: bias
        )

        let debugMessage = "Trying to find relays for \(obfuscationMethod) for the \(connectionAttemptCount)th attempt"
        print(debugMessage)
        return switch obfuscationMethod {
        case .udpOverTcp:
            UdpOverTcpObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).obfuscate()
        case .shadowsocks:
            ShadowsocksObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).obfuscate()
        case .quic:
            QuicObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).obfuscate()
        default:
            RelayObfuscation(
                allRelays: relays,
                obfuscatedRelays: relays,
                port: tunnelSettings.relayConstraints.port,
                method: obfuscationMethod
            )
        }
    }
}
