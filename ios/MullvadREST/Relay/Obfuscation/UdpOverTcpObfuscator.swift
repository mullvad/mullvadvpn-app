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

struct UdpOverTcpObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings

    func obfuscate() -> RelayObfuscation {
        RelayObfuscation(
            relays: relays,
            port: obfuscateUdpOverTcpPort(tunnelSettings: tunnelSettings),
            method: .udpOverTcp
        )
    }

    private func obfuscateUdpOverTcpPort(tunnelSettings: LatestTunnelSettings) -> RelayConstraint<UInt16> {
        switch tunnelSettings.wireGuardObfuscation.udpOverTcpPort {
        case .automatic:
            // Only include ports 80 and 443 for automatic case since they're the most
            // likely to work reliably.
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
