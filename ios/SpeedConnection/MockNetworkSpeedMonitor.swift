//
//  MockNetworkSpeedMonitor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation

final class MockNetworkSpeedMonitor: NetworkSpeedMonitorProtocol, @unchecked Sendable {
    
    var onUpdateTrafficSummery: (@Sendable (TrafficSummery) -> Void)?

    private var timer: Timer?

    func start(timeInterval: TimeInterval) {
        timer?.invalidate()

        timer = Timer.scheduledTimer(withTimeInterval: timeInterval, repeats: true) { [weak self] _ in
            guard let self else { return }

            let sent = UInt64.random(in: 10_000...100_000)
            let received = UInt64.random(in: 10_000...100_000)

            let new = TrafficPackage(wifi: TrafficData(sent: sent, received: received))

            self.onUpdateTrafficSummery?(
                TrafficSummery.make(self.previousTraffic, new: new, interval: timeInterval)
            )

            self.previousTraffic = new
        }
    }

    func stop() {
        timer?.invalidate()
        timer = nil
    }

    private var previousTraffic = TrafficPackage()
}
