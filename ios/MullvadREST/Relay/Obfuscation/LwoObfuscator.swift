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

struct LwoObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings

    func obfuscate() throws -> RelayObfuscation {
        RelayObfuscation(
            relays: filterLwoRelays(from: relays),
            port: try validateLwoPort(relays: relays, tunnelSettings: tunnelSettings),
            method: .lwo
        )
    }

    private func filterLwoRelays(from relays: REST.ServerRelaysResponse) -> REST.ServerRelaysResponse {
        return REST.ServerRelaysResponse(
            locations: relays.locations,
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: relays.wireguard.ipv4Gateway,
                ipv6Gateway: relays.wireguard.ipv6Gateway,
                portRanges: relays.wireguard.portRanges,
                relays: relays.wireguard.relays.filter { $0.supportsLwo },
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            ),
            bridge: relays.bridge
        )
    }

    private func validateLwoPort(
        relays: REST.ServerRelaysResponse,
        tunnelSettings: LatestTunnelSettings
    ) throws -> RelayConstraint<UInt16> {
        guard let customLwoPort = tunnelSettings.wireGuardObfuscation.lwoPort.portValue else {
            return .any
        }

        let portIsWithinValidWireGuardRanges = relays.wireguard.portRanges
            .contains { range in
                if let minPort = range.first, let maxPort = range.last {
                    return (minPort...maxPort).contains(customLwoPort)
                }
                return false
            }

        if !portIsWithinValidWireGuardRanges {
            throw NoRelaysSatisfyingConstraintsError(.invalidPort)
        }

        return .only(customLwoPort)
    }
}
