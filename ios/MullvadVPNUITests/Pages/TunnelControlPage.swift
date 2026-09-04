// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

class TunnelControlPage: Page {
    private struct ConnectionAttempt: Hashable {
        let ipAddress: String
        let port: String
        let protocolName: String
        /// Label of the obfuscation feature indicator, `nil` when no obfuscation is in use.
        let obfuscation: String?

        func isSameEndpoint(as other: ConnectionAttempt?) -> Bool {
            ipAddress == other?.ipAddress && port == other?.port && protocolName == other?.protocolName
        }
    }

    /// Labels of the obfuscation feature indicator, one per obfuscation method.
    enum ObfuscationIndicator {
        static let udpOverTcp = "UDP-over-TCP"
        static let shadowsocks = "Shadowsocks"
        static let quic = "QUIC"
        static let lwo = "LWO"
    }

    /// Strips the accessibility label prefix (e.g. "In ") from a combined row label,
    /// returning just the value portion (e.g. "85.203.53.104:56678 UDP").
    nonisolated static func connectionDetailValue(from label: String) -> String {
        let components = label.split(separator: " ")
        // The value starts at the first component containing ":" (ip:port) or "." (ip address)
        if let index = components.firstIndex(where: { $0.contains(":") || $0.contains(".") }) {
            return components[index...].joined(separator: " ")
        }
        return label
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
        let pollingInterval = TimeInterval(0.5)  // How often to check for changes

        let inAddressRow = app.staticTexts[AccessibilityIdentifier.connectionPanelInAddressRow]
        let obfuscationIndicator = app.buttons[AccessibilityIdentifier.obfuscationFeatureIndicator]

        while Date().timeIntervalSince(startTime) < timeout {
            let expectation = XCTestExpectation(description: "Wait for connection attempts")

            DispatchQueue.global().asyncAfter(deadline: .now() + pollingInterval) {
                expectation.fulfill()
            }

            _ = XCTWaiter.wait(for: [expectation], timeout: pollingInterval + 0.5)

            guard inAddressRow.exists else {
                continue
            }

            let currentText = Self.connectionDetailValue(from: inAddressRow.label)

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
                protocolName: protocolName,
                obfuscation: obfuscationIndicator.exists ? obfuscationIndicator.label : nil
            )

            if !connectionAttempt.isSameEndpoint(as: lastConnectionAttempt) {
                connectionAttempts.append(connectionAttempt)
                lastConnectionAttempt = connectionAttempt

                if connectionAttempts.count == attemptsCount {
                    break
                }
            }
        }

        return connectionAttempts
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

    @discardableResult func tapReconnectButton() -> Self {
        app.buttons[AccessibilityIdentifier.reconnectButton].tap()
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
            .existsAfterWait(timeout: .extremelyLong)
        XCTAssertTrue(labelFound, "Secure connection label not presented")

        return self
    }

    @discardableResult func tapRelayStatusExpandCollapseButton() -> Self {
        let button = app.otherElements[AccessibilityIdentifier.relayStatusCollapseButton]
        button.tapWhenHittable(failOnUnmetCondition: false)

        return self
    }

    /// Verify that connection attempts are made in the correct order
    @discardableResult func verifyConnectionAttemptsOrder() -> Self {
        // Number of connection attempts should be equal to the number of obfuscation methods (incl. "off").
        var connectionAttempts = waitForConnectionAttempts(5, timeout: 80)
        XCTAssertEqual(connectionAttempts.count, 5)

        // The automatic obfuscation order is off, Shadowsocks, QUIC, UDP-over-TCP, LWO.
        // No indicator is shown for the unobfuscated attempt.
        let expectedAttempts: [(protocolName: String, obfuscation: String?)] = [
            ("UDP", nil),
            ("UDP", ObfuscationIndicator.shadowsocks),
            ("UDP", ObfuscationIndicator.quic),
            ("TCP", ObfuscationIndicator.udpOverTcp),
            ("UDP", ObfuscationIndicator.lwo),
        ]
        for (attempt, expected) in zip(connectionAttempts, expectedAttempts) {
            XCTAssertEqual(attempt.protocolName, expected.protocolName)
            XCTAssertEqual(attempt.obfuscation, expected.obfuscation)
        }
        return self
    }

    /// Verify that the obfuscation feature indicator names the expected obfuscation method.
    /// Requires the connection panel to be expanded, otherwise the chip may be collapsed
    /// behind the "more..." button.
    @discardableResult func verifyObfuscationFeatureIndicator(_ expectedLabel: String) -> Self {
        let indicator = app.buttons[AccessibilityIdentifier.obfuscationFeatureIndicator]
        XCTAssertTrue(indicator.existsAfterWait(), "Obfuscation feature indicator not presented")
        XCTAssertEqual(indicator.label, expectedLabel)
        return self
    }

    @discardableResult func verifyConnectedRelays(entry: String, exit: String) -> Self {
        let serverLabel = app.staticTexts[.connectionPanelServerLabel]
        XCTAssertTrue(serverLabel.exists)
        XCTAssertTrue(
            serverLabel.label.contains(exit) && serverLabel.label.contains(entry),
            "Expected server label to contain '\(exit)' and '\(entry)', got '\(serverLabel.label)'"
        )
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
        XCTAssertTrue(app.buttons["Multihop"].exists)
        return self
    }

    /// Verify that the app attempts to connect using DAITA.
    @discardableResult func verifyConnectingUsingDAITA() -> Self {
        XCTAssertTrue(app.buttons["DAITA"].exists)
        return self
    }

    /// Verify that the app attempts to connect using DAITA.
    @discardableResult func verifyConnectingUsingDAITAThroughMultihop() -> Self {
        XCTAssertTrue(app.buttons["DAITA"].exists)
        XCTAssertTrue(app.buttons.images["IconMultihopWhenNeeded"].exists)
        return self
    }

    /// Verify that the app attempts to connect using quantum resistance.
    @discardableResult func verifyConnectingUsingQuantumResistance() -> Self {
        XCTAssertTrue(app.buttons["Quantum resistance"].exists)
        return self
    }

    /// Verify that the app does not attempt to connect using quantum resistance.
    @discardableResult func verifyNotConnectingUsingQuantumResistance() -> Self {
        XCTAssertTrue(app.buttons["Quantum resistance"].notExistsAfterWait())
        return self
    }

    @discardableResult func verifyConnectingUsingIncludeAllNetworks() -> Self {
        XCTAssertTrue(app.buttons["Force all apps"].existsAfterWait())
        return self
    }

    @discardableResult func verifyNotConnectingUsingIncludeAllNetworks() -> Self {
        XCTAssertTrue(app.buttons["Force all apps"].notExistsAfterWait())
        return self
    }

    @discardableResult func verifyConnectingUsingLocalNetworkSharing() -> Self {
        XCTAssertTrue(app.buttons["Local network sharing"].existsAfterWait())
        return self
    }

    @discardableResult func verifyNotConnectingUsingLocalNetworkSharing() -> Self {
        XCTAssertTrue(app.buttons["Local network sharing"].notExistsAfterWait())
        return self
    }

    @discardableResult func verifyFeatureIndicatorVisible(feature: String) -> Self {
        XCTAssertTrue(app.buttons[feature].existsAfterWait())
        return self
    }

    func getInIPAddressAndPortFromConnectionStatus() -> (String, Int) {
        let inAddressRow = app.staticTexts[.connectionPanelInAddressRow]
        // The combined row label looks like "In, 85.203.53.145:43030 UDP"
        let value = Self.connectionDetailValue(from: inAddressRow.label)
        let components = value.components(separatedBy: ":")
        let inIpAddress = components[0]  // 85.203.53.145
        let inPort = components[1].components(separatedBy: " ")[0]  // 43030
        return (inIpAddress, Int(inPort)!)
    }

    func getCurrentRelayName() -> String {
        let server = app.staticTexts[.connectionPanelServerLabel]
        return server.label
    }
}
