//
//  PingStats.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2024-02-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Ping statistics.
public struct PingStats {
    /// Dictionary holding sequence and corresponding date when echo request took place.
    var requests = [UInt16: Date]()

    /// Timestamp when last echo request was sent.
    var lastRequestDate: Date?

    /// Timestamp when last echo reply was received.
    var lastReplyDate: Date?

    public init(requests: [UInt16: Date] = [UInt16: Date](), lastRequestDate: Date? = nil, lastReplyDate: Date? = nil) {
        self.requests = requests
        self.lastRequestDate = lastRequestDate
        self.lastReplyDate = lastReplyDate
    }
}
