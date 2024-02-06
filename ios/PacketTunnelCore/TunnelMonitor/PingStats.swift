//
//  PingStats.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2024-02-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Ping statistics.
struct PingStats {
    /// Dictionary holding sequence and corresponding date when echo request took place.
    var requests = [UInt16: Date]()

    /// Timestamp when last echo request was sent.
    var lastRequestDate: Date?

    /// Timestamp when last echo reply was received.
    var lastReplyDate: Date?
}
