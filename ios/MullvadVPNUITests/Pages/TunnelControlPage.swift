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

        self.pageAccessibilityIdentifier = .tunnelControlView
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

    @discardableResult func waitForSecureConnectionLabel() -> Self {
        _ = app.staticTexts[AccessibilityIdentifier.connectionStatusConnectedLabel]
            .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
        return self
    }

    @discardableResult func tapRelayStatusExpandCollapseButton() -> Self {
        app.buttons[AccessibilityIdentifier.relayStatusCollapseButton].tap()
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
}
