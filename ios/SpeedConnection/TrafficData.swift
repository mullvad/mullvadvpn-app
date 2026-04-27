//
//  TrafficData.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//


struct TrafficData {
    var sent: UInt64  = 0// in bytes
    var received: UInt64 = 0 // in bytes
}

extension TrafficData {
    static var zero: TrafficData {
        TrafficData()
    }
    
    var isZero: Bool {
          received == 0 && sent == 0
      }
}
func +(lhs: TrafficData, rhs: TrafficData) -> TrafficData {
    var result = lhs
    result.received = lhs.received &+ rhs.received
    result.sent = lhs.sent &+ rhs.sent
    return result
}
