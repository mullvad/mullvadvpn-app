//
//  TunnelMonitorConfiguration.swift
//  PacketTunnel
//
//  Created by pronebird on 10/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum TunnelMonitorConfiguration {
    /// Interval at which to query the adapter for stats.
    static let wgStatsQueryInterval: DispatchTimeInterval = .milliseconds(50)

    /// Interval for sending echo packets.
    static let pingInterval: DispatchTimeInterval = .seconds(3)

    /// Delay before sending the first echo packet.
    static let pingStartDelay: DispatchTimeInterval = .milliseconds(500)

    /// Interval after which connection is treated as being lost.
    static let connectionTimeout: TimeInterval = 15
}
