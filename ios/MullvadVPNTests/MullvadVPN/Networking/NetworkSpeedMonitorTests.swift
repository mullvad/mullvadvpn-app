//
//  NetworkSpeedMonitorTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Testing

@Suite("NetworkSpeedMonitorTests")
struct NetworkSpeedMonitorTests {

    @Test
    func start() {
        let monitor = NetworkSpeedMonitor()
        monitor.start()

    }
}
