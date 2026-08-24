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
import PacketTunnelCore

/// Tunnel device stub that returns fixed interface name and feeds network stats from the type implementing `NetworkStatsProviding`
struct TunnelDeviceInfoStub: TunnelDeviceInfoProtocol, @unchecked Sendable {
    let networkStatsProviding: NetworkStatsProviding

    var interfaceName: String? {
        return "utun0"
    }

    func getStats() throws -> WgStats {
        return WgStats(
            bytesReceived: networkStatsProviding.bytesReceived,
            bytesSent: networkStatsProviding.bytesSent
        )
    }
}
