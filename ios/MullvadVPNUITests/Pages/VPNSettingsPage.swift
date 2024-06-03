//
//  VPNSettingsPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class VPNSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    private func cellExpandButton(_ cellAccessiblityIdentifier: AccessibilityIdentifier) -> XCUIElement {
        let table = app.tables[AccessibilityIdentifier.vpnSettingsTableView]
        let matchingCells = table.otherElements.containing(.any, identifier: cellAccessiblityIdentifier.rawValue)
        let expandButton = matchingCells.buttons[AccessibilityIdentifier.expandButton]

        return expandButton
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func tapDNSSettingsCell() -> Self {
        app.tables
            .cells[AccessibilityIdentifier.dnsSettings]
            .tap()

        return self
    }

    @discardableResult func tapWireGuardPortsExpandButton() -> Self {
        cellExpandButton(AccessibilityIdentifier.wireGuardPortsCell).tap()
        return self
    }

    @discardableResult func tapWireGuardObfuscationExpandButton() -> Self {
        cellExpandButton(AccessibilityIdentifier.wireGuardObfuscationCell).tap()

        return self
    }

    @discardableResult func tapUDPOverTCPPortExpandButton() -> Self {
        cellExpandButton(AccessibilityIdentifier.udpOverTCPPortCell).tap()

        return self
    }

    @discardableResult func tapUDPOverTCPPortAutomaticCell() -> Self {
        app.cells["\(AccessibilityIdentifier.wireGuardObfuscationPort)Automatic"]
            .tap()
        return self
    }

    @discardableResult func tapUDPOverTCPPort80Cell() -> Self {
        app.cells["\(AccessibilityIdentifier.wireGuardObfuscationPort)80"]
            .tap()
        return self
    }

    @discardableResult func tapUDPOverTCPPort5001Cell() -> Self {
        app.cells["\(AccessibilityIdentifier.wireGuardObfuscationPort)5001"]
            .tap()
        return self
    }

    @discardableResult func tapQuantumResistantTunnelExpandButton() -> Self {
        cellExpandButton(AccessibilityIdentifier.quantumResistantTunnelCell).tap()

        return self
    }

    @discardableResult func tapQuantumResistantTunnelAutomaticCell() -> Self {
        app.cells[AccessibilityIdentifier.quantumResistanceAutomatic]
            .tap()
        return self
    }

    @discardableResult func tapQuantumResistantTunnelOnCell() -> Self {
        app.cells[AccessibilityIdentifier.quantumResistanceOn]
            .tap()
        return self
    }

    @discardableResult func tapQuantumResistantTunnelOffCell() -> Self {
        app.cells[AccessibilityIdentifier.quantumResistanceOff]
            .tap()
        return self
    }

    @discardableResult func tapWireGuardObfuscationAutomaticCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationAutomatic]
            .tap()
        return self
    }

    @discardableResult func tapWireGuardObfuscationOnCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationOn].tap()

        return self
    }

    @discardableResult func tapWireGuardObfuscationOffCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationOff].tap()

        return self
    }

    @discardableResult func tapCustomWireGuardPortTextField() -> Self {
        app.textFields[AccessibilityIdentifier.customWireGuardPortTextField]
            .tap()
        return self
    }

    @discardableResult func verifyCustomWireGuardPortSelected(portNumber: String) -> Self {
        let cell = app.cells[AccessibilityIdentifier.wireGuardCustomPort]
        XCTAssertTrue(cell.isSelected)
        let textField = app.textFields[AccessibilityIdentifier.customWireGuardPortTextField]

        guard let textFieldValue = textField.value as? String else {
            XCTFail("Failed to read custom port text field value")
            return self
        }

        XCTAssertEqual(textFieldValue, portNumber)
        return self
    }

    @discardableResult func verifyWireGuardObfuscationOnSelected() -> Self {
        let onCell = app.cells[AccessibilityIdentifier.wireGuardObfuscationOn]
        XCTAssertTrue(onCell.isSelected)
        return self
    }

    @discardableResult func verifyUDPOverTCPPort80Selected() -> Self {
        let cell = app.cells["\(AccessibilityIdentifier.wireGuardObfuscationPort)80"]
        XCTAssertTrue(cell.isSelected)
        return self
    }

    @discardableResult func verifyQuantumResistantTunnelOffSelected() -> Self {
        let cell = app.cells[AccessibilityIdentifier.quantumResistanceOff]
        XCTAssertTrue(cell.isSelected)
        return self
    }
}
