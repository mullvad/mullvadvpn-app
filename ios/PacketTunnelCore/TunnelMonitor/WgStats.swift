//
//  WgStats.swift
//  PacketTunnelCore
//
//  Created by pronebird on 08/08/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct WgStats: Sendable {
    public let bytesReceived: UInt64
    public let bytesSent: UInt64

    public init(bytesReceived: UInt64 = 0, bytesSent: UInt64 = 0) {
        self.bytesReceived = bytesReceived
        self.bytesSent = bytesSent
    }
}
