//
//  TunnelControlPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class TunnelControlPage: Page {
    private struct ConnectionAttempt: Hashable {
        let ipAddress: String
        let port: String
        let protocolName: String
    }

    var connectionIsSecured: Bool {
        app.staticTexts[AccessibilityIdentifier.connectionStatusConnectedLabel].exists
    }

    /// Poll the "in address row" label for its updated values and output an array of ConnectionAttempt objects representing the connection attempts that have been communicated through the UI.
    /// - Parameters:
    ///   - attemptsCount: number of connection attempts to look for
    ///   - timeout: return the attemps found so far after this many seconds if `attemptsCount` haven't been reached yet
    private func waitForConnectionAttempts(_ attemptsCount: Int, timeout: TimeInterval) -> [ConnectionAttempt] {
        var connectionAttempts: [ConnectionAttempt] = []
        var lastConnectionAttempt: ConnectionAttempt?
        let startTime = Date()
        let pollingInterval = TimeInterval(0.5) // How often to check for changes

        let inAddressRow = app.staticTexts[AccessibilityIdentifier.connectionPanelInAddressRow]

        while Date().timeIntervalSince(startTime) < timeout {
            let expectation = XCTestExpectation(description: "Wait for connection attempts")

            DispatchQueue.global().asyncAfter(deadline: .now() + pollingInterval) {
                expectation.fulfill()
            }

            _ = XCTWaiter.wait(for: [expectation], timeout: pollingInterval + 0.5)

            let currentText = inAddressRow.label

            // Skip initial label value with IP address only - no port or protocol
            guard currentText.contains(" ") == true else {
                continue
            }

            let addressPortComponent = currentText.components(separatedBy: " ")[0]
            let ipAddress = addressPortComponent.components(separatedBy: ":")[0]
            let port = addressPortComponent.components(separatedBy: ":")[1]
            let protocolName = currentText.components(separatedBy: " ")[1]
            let connectionAttempt = ConnectionAttempt(
                ipAddress: ipAddress,
                port: port,
                protocolName: protocolName
            )

            if connectionAttempt != lastConnectionAttempt {
                connectionAttempts.append(connectionAttempt)
                lastConnectionAttempt = connectionAttempt

                if connectionAttempts.count == attemptsCount {
                    break
                }
            }
        }

        return connectionAttempts
    }

    func getInIPv4AddressLabel() -> String {
        app.staticTexts[AccessibilityIdentifier.connectionPanelInAddressRow].label.components(separatedBy: ":")[0]
    }

    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.connectionView]
        waitForPageToBeShown()
    }

    @discardableResult func tapSelectLocationButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationButton].tap()
        return self
    }

    @discardableResult func tapConnectButton() -> Self {
        app.buttons[AccessibilityIdentifier.connectButton].tap()
        return self
    }

    @discardableResult func tapDisconnectButton() -> Self {
        app.buttons[AccessibilityIdentifier.disconnectButton].tap()
        return self
    }

    @discardableResult func tapCancelButton() -> Self {
        app.buttons[AccessibilityIdentifier.cancelButton].tap()
        return self
    }

    /// Tap either cancel or disconnect button, depending on the current connection state. Use this function sparingly when it's irrelevant whether the app is currently connecting to a relay or already connected.
    @discardableResult func tapCancelOrDisconnectButton() -> Self {
        let cancelButton = app.buttons[.cancelButton]
        let disconnectButton = app.buttons[.disconnectButton]

        if disconnectButton.exists && disconnectButton.isHittable {
            disconnectButton.tap()
        } else {
            cancelButton.tap()
        }

        return self
    }

    @discardableResult func waitForConnectedLabel() -> Self {
        let labelFound = app.staticTexts[.connectionStatusConnectedLabel]
            .waitForExistence(timeout: BaseUITestCase.extremelyLongTimeout)
        XCTAssertTrue(labelFound, "Secure connection label presented")

        return self
    }

    @discardableResult func tapRelayStatusExpandCollapseButton() -> Self {
        app.buttons[AccessibilityIdentifier.relayStatusCollapseButton].tap()
        return self
    }

    /// Verify that the app attempts to connect over UDP before switching to TCP. For testing blocked UDP traffic.
    @discardableResult func verifyConnectingOverTCPAfterUDPAttempts() -> Self {
        let connectionAttempts = waitForConnectionAttempts(4, timeout: 30)

        // Should do four connection attempts but due to UI bug sometimes only two are displayed, sometimes all four
        if connectionAttempts.count == 4 { // Expected retries flow
            for (attemptIndex, attempt) in connectionAttempts.enumerated() {
                if attemptIndex < 3 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 3 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                } else {
                    XCTFail("Unexpected connection attempt")
                }
            }
        } else if connectionAttempts.count == 3 { // Most of the times this incorrect flow is shown
            for (attemptIndex, attempt) in connectionAttempts.enumerated() {
                if attemptIndex == 0 || attemptIndex == 1 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 2 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                } else {
                    XCTFail("Unexpected connection attempt")
                }
            }
        } else {
            XCTFail("Unexpected number of connection attempts, expected 3~4, got \(connectionAttempts.count)")
        }

        return self
    }

    /// Verify that connection attempts are made in the correct order
    @discardableResult func verifyConnectionAttemptsOrder() -> Self {
        var connectionAttempts = waitForConnectionAttempts(4, timeout: 80)
        var totalAttemptsOffset = 0
        XCTAssertEqual(connectionAttempts.count, 4)

        /// Sometimes, the UI will only show an IP address for the first connection attempt, which gets skipped by
        /// `waitForConnectionAttempts`, and offsets expected attempts count by 1, but still counts towards
        /// total connection attempts. Remove that last attempt which would be the first one of a new series
        /// of connection attempts.
        if connectionAttempts.last?.protocolName == "UDP" {
            connectionAttempts.removeLast()
            totalAttemptsOffset = 1
        }
        for (attemptIndex, attempt) in connectionAttempts.enumerated() {
            if attemptIndex < 3 - totalAttemptsOffset {
                XCTAssertEqual(attempt.protocolName, "UDP")
            } else {
                XCTAssertEqual(attempt.protocolName, "TCP")
                let validPorts = ["80", "5001"]
                XCTAssertTrue(validPorts.contains(attempt.port))
            }
        }

        return self
    }

    @discardableResult func verifyConnectingToPort(_ port: String) -> Self {
        let connectionAttempts = waitForConnectionAttempts(1, timeout: 10)
        XCTAssertEqual(connectionAttempts.count, 1)
        XCTAssertEqual(connectionAttempts.first!.port, port)

        return self
    }

    /// Verify that the app attempts to connect over Multihop.
    @discardableResult func verifyConnectingOverMultihop() -> Self {
        XCTAssertTrue(app.staticTexts["Multihop"].exists)
        return self
    }

    /// Verify that the app attempts to connect using DAITA.
    @discardableResult func verifyConnectingUsingDAITA() -> Self {
        XCTAssertTrue(app.staticTexts["DAITA"].exists)
        return self
    }

    func getInIPAddressFromConnectionStatus() -> String {
        let inAddressRow = app.staticTexts[.connectionPanelInAddressRow]
        return inAddressRow.label.components(separatedBy: ":")[0]
    }

    func getCurrentRelayName() -> String {
        let server = app.staticTexts[.connectionPanelServerLabel]
        return server.label
    }
}
