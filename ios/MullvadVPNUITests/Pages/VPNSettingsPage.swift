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

class VPNSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
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
        app.buttons[.ipVersionCell].tap()
        return self
    }

    @discardableResult func tapIPVersionAutomaticCell() -> Self {
        app.buttons[.ipVersionAutomatic].tap()
        return self
    }

    @discardableResult func tapIPVersionIPv4Cell() -> Self {
        app.buttons[.ipVersionIPv4].tap()
        return self
    }

    @discardableResult func tapIPVersionIPv6Cell() -> Self {
        app.buttons[.ipVersionIPv6].tap()
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
