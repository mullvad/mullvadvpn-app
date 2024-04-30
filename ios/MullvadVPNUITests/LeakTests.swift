//
//  LeakTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-05-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class LeakTests: LoggedInWithTimeUITestCase {
    /// For now just the skeleton of a leak test - traffic is captured and parsed, but not analyzed
    func testLeaks() throws {
        startPacketCapture()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        // Trigger traffic by navigating to website in Safari
        let safariApp = SafariApp()
        safariApp.launch()
        safariApp.tapAddressBar()
        safariApp.enterText("mullvad.net\n")

        app.launch()
        TunnelControlPage(app)
            .tapDisconnectButton()

        // Keep the capture open for a while
        Thread.sleep(forTimeInterval: 5.0)
        let capturedTraffic = stopPacketCapture()

        // Analyze captured traffic
    }
}
