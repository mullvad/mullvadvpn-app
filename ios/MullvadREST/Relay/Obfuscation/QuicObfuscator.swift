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

struct QuicObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings

    func obfuscate() -> RelayObfuscation {
        RelayObfuscation(
            relays: filterQuicRelays(from: relays),
            port: .only(443),
            method: .quic
        )
    }

    private func filterQuicRelays(from relays: REST.ServerRelaysResponse) -> REST.ServerRelaysResponse {
        var filteredRelays = relays.wireguard.relays.filter { $0.supportsQuic }

        // If IPv6 is required, filter to relays with QUIC IPv6 addresses
        // Regular entry IPv6 addresses don't work with QUIC
        if tunnelSettings.ipVersion.isIPv6 {
            filteredRelays = filteredRelays.filter { $0.hasQuicIpv6 }
        } else {
            filteredRelays = filteredRelays.filter { $0.hasQuicIpv4 }
        }

        return REST.ServerRelaysResponse(
            locations: relays.locations,
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: relays.wireguard.ipv4Gateway,
                ipv6Gateway: relays.wireguard.ipv6Gateway,
                portRanges: relays.wireguard.portRanges,
                relays: filteredRelays,
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            ),
            bridge: relays.bridge
        )
    }
}
