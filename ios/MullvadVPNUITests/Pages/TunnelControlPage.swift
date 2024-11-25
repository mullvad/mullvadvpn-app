//
//  TunnelControlPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

        let inAddressRow = app.otherElements[AccessibilityIdentifier.connectionPanelInAddressRow]

        while Date().timeIntervalSince(startTime) < timeout {
            let expectation = XCTestExpectation(description: "Wait for connection attempts")

            DispatchQueue.global().asyncAfter(deadline: .now() + pollingInterval) {
                expectation.fulfill()
            }

            _ = XCTWaiter.wait(for: [expectation], timeout: pollingInterval + 0.5)

            if let currentText = inAddressRow.value as? String {
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
        }

        return connectionAttempts
    }

    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.tunnelControlView]
        waitForPageToBeShown()
    }

    @discardableResult func tapSelectLocationButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationButton].tap()
        return self
    }

    @discardableResult func tapSecureConnectionButton() -> Self {
        app.buttons[AccessibilityIdentifier.secureConnectionButton].tap()
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

    @discardableResult func waitForSecureConnectionLabel() -> Self {
        let labelFound = app.staticTexts[.connectionStatusConnectedLabel]
            .waitForExistence(timeout: BaseUITestCase.extremelyLongTimeout)
        XCTAssertTrue(labelFound, "Secure connection label presented")

        return self
    }

    @discardableResult func tapRelayStatusExpandCollapseButton() -> Self {
        app.otherElements[AccessibilityIdentifier.relayStatusCollapseButton].press(forDuration: .leastNonzeroMagnitude)
        return self
    }

    /// Verify that the app attempts to connect over UDP before switching to TCP. For testing blocked UDP traffic.
    @discardableResult func verifyConnectingOverTCPAfterUDPAttempts() -> Self {
        let connectionAttempts = waitForConnectionAttempts(3, timeout: 15)

        // Should do three connection attempts but due to UI bug sometimes only two are displayed, sometimes all three
        if connectionAttempts.count == 3 { // Expected retries flow
            for (attemptIndex, attempt) in connectionAttempts.enumerated() {
                if attemptIndex == 0 || attemptIndex == 1 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 2 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                } else {
                    XCTFail("Unexpected connection attempt")
                }
            }
        } else if connectionAttempts.count == 2 { // Most of the times this incorrect flow is shown
            for (attemptIndex, attempt) in connectionAttempts.enumerated() {
                if attemptIndex == 0 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 1 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                } else {
                    XCTFail("Unexpected connection attempt")
                }
            }
        } else {
            XCTFail("Unexpected number of connection attempts")
        }

        return self
    }

    /// Verify that connection attempts are made in the correct order
    @discardableResult func verifyConnectionAttemptsOrder() -> Self {
        let connectionAttempts = waitForConnectionAttempts(4, timeout: 50)
        XCTAssertEqual(connectionAttempts.count, 4)

        if connectionAttempts.last?.protocolName == "UDP" {
            // If last attempt is over UDP it means we have encountered the UI bug where only one UDP attempt is shown and then the two TCP attempts
            for (attemptIndex, attempt) in connectionAttempts.enumerated() {
                if attemptIndex == 0 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 1 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                    XCTAssertEqual(attempt.port, "80")
                } else if attemptIndex == 2 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                    XCTAssertEqual(attempt.port, "5001")
                } // Ignore the 4th attempt which is the first attempt of new attempt cycle
            }
        } else {
            for (attemptIndex, attempt) in connectionAttempts.enumerated() {
                if attemptIndex == 0 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 1 {
                    XCTAssertEqual(attempt.protocolName, "UDP")
                } else if attemptIndex == 2 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                    XCTAssertEqual(attempt.port, "80")
                } else if attemptIndex == 3 {
                    XCTAssertEqual(attempt.protocolName, "TCP")
                    XCTAssertEqual(attempt.port, "5001")
                }
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
        let relayName = getCurrentRelayName().lowercased()
        XCTAssertTrue(relayName.contains("via"))
        return self
    }

    /// Verify that the app attempts to connect using DAITA.
    @discardableResult func verifyConnectingUsingDAITA() -> Self {
        let relayName = getCurrentRelayName().lowercased()
        XCTAssertTrue(relayName.contains("using DAITA"))
        return self
    }

    func getInIPAddressFromConnectionStatus() -> String {
        let inAddressRow = app.otherElements[AccessibilityIdentifier.connectionPanelInAddressRow]

        if let textValue = inAddressRow.value as? String {
            let ipAddress = textValue.components(separatedBy: ":")[0]
            return ipAddress
        } else {
            XCTFail("Failed to read relay IP address from status label")
            return String()
        }
    }

    func getCurrentRelayName() -> String {
        let relayExpandButton = app.otherElements[.relayStatusCollapseButton]

        guard let relayName = relayExpandButton.value as? String else {
            XCTFail("Failed to read relay name from tunnel control page")
            return String()
        }

        return relayName
    }
}
