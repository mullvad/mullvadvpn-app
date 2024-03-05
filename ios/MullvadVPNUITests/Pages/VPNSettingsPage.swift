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

    private func cellExpandCollapseButton(_ cellAccessiblityIdentifier: AccessibilityIdentifier) -> XCUIElement {
        let table = app.tables[AccessibilityIdentifier.vpnSettingsTableView]
        let matchingCells = table.otherElements.containing(.any, identifier: cellAccessiblityIdentifier.rawValue)
        let expandButton = matchingCells.buttons[AccessibilityIdentifier.collapseButton]

        return expandButton
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func tapWireGuardPortsExpandButton() -> Self {
        cellExpandCollapseButton(AccessibilityIdentifier.wireGuardPortsCell).tap()

        return self
    }

    @discardableResult func tapWireGuardObfuscationExpandButton() -> Self {
        cellExpandCollapseButton(AccessibilityIdentifier.wireGuardObfuscationCell).tap()

        return self
    }

    @discardableResult func tapUDPOverTCPPortExpandButton() -> Self {
        cellExpandCollapseButton(AccessibilityIdentifier.udpOverTCPPortCell).tap()

        return self
    }

    @discardableResult func tapQuantumResistantTunnelExpandButton() -> Self {
        cellExpandCollapseButton(AccessibilityIdentifier.quantumResistantTunnelCell).tap()

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
        app.textFields[AccessibilityIdentifier.customWireGuardPortTextField].tap()

        return self
    }
}
