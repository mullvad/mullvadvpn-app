//
//  TunnelDeviceInfoStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

/// Tunnel device stub that returns fixed interface name and feeds network stats from the type implementing `NetworkStatsProviding`
struct TunnelDeviceInfoStub: TunnelDeviceInfoProtocol {
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
