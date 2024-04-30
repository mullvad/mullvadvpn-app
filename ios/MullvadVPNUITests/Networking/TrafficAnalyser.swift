//
//  TrafficAnalyser.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-06-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum LeakStatus {
    case noLeak
    case leak
    case acceptableLeak
    case unknown
}

class TrafficAnalyser {
    let capturedTraffic: [Stream]
    let testDeviceIPAddress: String
    let graceBuffer: TimeInterval = 0.3
    var vpnConnectionStartTime: Date?
    var vpnConnectionEndTime: Date?

    init(capturedTraffic: [Stream], testDeviceIPAddress: String) {
        self.capturedTraffic = capturedTraffic
        self.testDeviceIPAddress = testDeviceIPAddress

        self.determineVPNConnectionTimeframe()
    }

    private func determineVPNConnectionTimeframe() {
        vpnConnectionStartTime = Date()
        vpnConnectionEndTime = Date()
    }

    private func flagRelayTrafficAsOkay() {}

    private func flagAppleLeaksAsAcceptable() {}

    private func flagMDNSLeaksAsAcceptable() {}
}
