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
        let expandButton = matchingCells.buttons[AccessibilityIdentifier.collapseButton]

        return expandButton
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
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

    @discardableResult func tapQuantumResistantTunnelExpandButton() -> Self {
        cellExpandButton(AccessibilityIdentifier.quantumResistantTunnelCell).tap()

        return self
    }

    @discardableResult func tapWireGuardObfuscationValueAutomaticCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationAutomatic]
            .tap()

        return self
    }

    @discardableResult func tapWireGuardObfuscationValueOnCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationOn].tap()

        return self
    }

    @discardableResult func tapWireGuardObfuscationValueOffCell() -> Self {
        app.cells[AccessibilityIdentifier.wireGuardObfuscationOff].tap()

        return self
    }

    @discardableResult func tapCustomWireGuardPortTextField() -> Self {
        app.textFields[AccessibilityIdentifier.customWireGuardPortTextField].tap()

        return self
    }
}
