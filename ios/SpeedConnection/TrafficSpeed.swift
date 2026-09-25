//
//  TrafficSpeed.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct TrafficSpeed {
    /// bits per second
    var received: Double
    var sent: Double

    init(received: Double, sent: Double) {
        self.received = received
        self.sent = sent
    }

    public init(old: TrafficData, new: TrafficData, interval: TimeInterval) {
        guard interval > 0 else {
            self = .zero
            return
        }

        let deltaReceivedBytes = Double(new.received &- old.received)
        let deltaSentBytes = Double(new.sent &- old.sent)

        // bytes → bits → bits per second
        let receivedBps = (deltaReceivedBytes * 8.0) / interval
        let sentBps = (deltaSentBytes * 8.0) / interval

        self.received = receivedBps.isFinite ? receivedBps : 0
        self.sent = sentBps.isFinite ? sentBps : 0
    }
}
extension TrafficSpeed {
    static var zero: TrafficSpeed {
        .init(received: 0, sent: 0)
    }
    var isZero: Bool {
        received == 0 && sent == 0
    }
}

func + (lhs: TrafficSpeed, rhs: TrafficSpeed) -> TrafficSpeed {
    var result = lhs
    result.received += rhs.received
    result.sent += rhs.sent
    return result
}
