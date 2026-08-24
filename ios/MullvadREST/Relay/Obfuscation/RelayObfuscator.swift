// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes

protocol RelayObfuscating {
    var relays: REST.ServerRelaysResponse { get }
    var tunnelSettings: LatestTunnelSettings { get }
    func obfuscate() throws -> RelayObfuscation
}

public struct RelayObfuscation {
    let relays: REST.ServerRelaysResponse
    let port: RelayConstraint<UInt16>
    var method: WireGuardObfuscationState
}

struct RelayObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt
    let obfuscationBypass: any ObfuscationProviding

    func obfuscate() throws -> RelayObfuscation {
        let obfuscationMethod = ObfuscationMethodSelector.obfuscationMethodBy(
            connectionAttemptCount: connectionAttemptCount,
            tunnelSettings: tunnelSettings,
            obfuscationBypass: obfuscationBypass
        )

        return switch obfuscationMethod {
        case .udpOverTcp:
            UdpOverTcpObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings
            ).obfuscate()
        case .shadowsocks:
            ShadowsocksObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings
            ).obfuscate()
        case .quic:
            QuicObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings
            ).obfuscate()
        case .lwo:
            try LwoObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings
            ).obfuscate()
        default:
            RelayObfuscation(
                relays: relays,
                port: tunnelSettings.relayConstraints.port,
                method: obfuscationMethod
            )
        }
    }
}
