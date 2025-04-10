//
//  VPNSettingsPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class VPNSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    private func cellSubButton(
        _ cellAccessiblityIdentifier: AccessibilityIdentifier,
        _ subButtonAccessibilityIdentifier: AccessibilityIdentifier
    ) -> XCUIElement {
        let tableView = app.tables[AccessibilityIdentifier.vpnSettingsTableView]
        let matchingCells = tableView.cells[cellAccessiblityIdentifier]
        let expandButton = matchingCells.buttons[subButtonAccessibilityIdentifier]
        let lastCell = tableView.cells.allElementsBoundByIndex.last!
        tableView.scrollDownToElement(element: lastCell)
        return expandButton
    }

    private func cellExpandButton(_ cellAccessiblityIdentifier: AccessibilityIdentifier) -> XCUIElement {
        let tableView = app.tables[AccessibilityIdentifier.vpnSettingsTableView]
        let matchingCells = tableView.otherElements[cellAccessiblityIdentifier]
        let expandButton = matchingCells.buttons[.expandButton]
        let lastCell = tableView.cells.allElementsBoundByIndex.last!
        tableView.scrollDownToElement(element: lastCell)
        return expandButton
    }

    private func cellPortSelectorButton(_ cellAccessiblityIdentifier: AccessibilityIdentifier) -> XCUIElement {
        return cellSubButton(cellAccessiblityIdentifier, .openPortSelectorMenuButton)
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

    @discardableResult func tapLocalNetworkSharingSwitch() -> Self {
        app.cells[AccessibilityIdentifier.localNetworkSharing]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()
        app.buttons[AccessibilityIdentifier.acceptLocalNetworkSharingButton]
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

    @discardableResult func tapUDPOverTCPPortSelectorButton() -> Self {
        cellPortSelectorButton(AccessibilityIdentifier.wireGuardObfuscationUdpOverTcp).tap()

        return self
    }

    @discardableResult func tapShadowsocksPortSelectorButton() -> Self {
        cellPortSelectorButton(AccessibilityIdentifier.wireGuardObfuscationShadowsocks).tap()

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

    @discardableResult func tapWireGuardObfuscationUdpOverTcpCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationUdpOverTcp].tap()

        return self
    }

    @discardableResult func tapWireGuardObfuscationShadowsocksCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationShadowsocks].tap()

        return self
    }

    @discardableResult func tapWireGuardObufscationQuicCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationQuic].tap()
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
        let onCell = app.cells[AccessibilityIdentifier.wireGuardObfuscationUdpOverTcp]
        XCTAssertTrue(onCell.isSelected)
        return self
    }

    @discardableResult func verifyUDPOverTCPPort80Selected() -> Self {
        let detailLabel = app.staticTexts[AccessibilityIdentifier.wireGuardObfuscationUdpOverTcpPort]
        XCTAssertTrue(detailLabel.label.hasSuffix(" 80"))
        return self
    }

    @discardableResult func verifyQuantumResistantTunnelOffSelected() -> Self {
        let cell = app.cells[AccessibilityIdentifier.quantumResistanceOff]
        XCTAssertTrue(cell.isSelected)
        return self
    }

    @discardableResult func verifyQuantumResistantTunnelOnSelected() -> Self {
        let cell = app.cells[AccessibilityIdentifier.quantumResistanceOn]
        XCTAssertTrue(cell.isSelected)
        return self
    }
}
