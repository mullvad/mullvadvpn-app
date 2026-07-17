//
//  VPNSettingsPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
        return expandButton.wait()
    }

    private func cellPortSelectorButton(_ cellAccessiblityIdentifier: AccessibilityIdentifier) -> XCUIElement {
        cellSubButton(cellAccessiblityIdentifier, .openPortSelectorMenuButton)
    }

    private func quantumResistanceSwitch() -> XCUIElement {
        app.switches.allElementsBoundByIndex.last!
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func tapDNSSettingsCell() -> Self {
        app.buttons[.dnsSettings].tap()
        return self
    }

    @discardableResult func tapWireGuardPorts() -> Self {
        app.buttons[.wireGuardPorts].tap()
        return self
    }

    @discardableResult func tapAntiCensorshipCell() -> Self {
        app.buttons[.antiCensorship].tap()
        return self
    }

    /// Temporary workaround until the accessibility issues in `SegmentedListItem` are fixed
    @discardableResult func turnQuantumResistanceOn() -> Self {
        let quantumResistanceSwitch = quantumResistanceSwitch()
        if quantumResistanceSwitch.value as? String == "0" {
            quantumResistanceSwitch.tap()
        }
        return self
    }

    /// Temporary workaround until the accessibility issues in `SegmentedListItem` are fixed
    @discardableResult func turnQuantumResistanceOff() -> Self {
        let quantumResistanceSwitch = quantumResistanceSwitch()
        if quantumResistanceSwitch.value as? String == "1" {
            quantumResistanceSwitch.tap()
        }
        return self
    }

    @discardableResult func tapIPVersionExpandButton() -> Self {
        cellExpandButton(.ipVersionCell).tap()

        return self
    }

    @discardableResult func tapIPVersionAutomaticCell() -> Self {
        app.cells[.ipVersionAutomatic]
            .tap()
        return self
    }

    @discardableResult func tapIPVersionIPv4Cell() -> Self {
        app.cells[.ipVersionIPv4]
            .tap()
        return self
    }

    @discardableResult func tapIPVersionIPv6Cell() -> Self {
        app.cells[.ipVersionIPv6]
            .tap()
        return self
    }

    @discardableResult func tapCustomWireGuardPortTextField() -> Self {
        app.textFields[.customWireGuardPortTextField]
            .tap()
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
