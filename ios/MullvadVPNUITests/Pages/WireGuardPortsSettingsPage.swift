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

class WireGuardPortsSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    private var table: XCUIElement {
        app.collectionViews[AccessibilityIdentifier.wireGuardObfuscationPortTable]
    }

    private func portCell(_ index: Int) -> XCUIElement {
        table.cells.element(boundBy: index)
    }

    private var customCell: XCUIElement {
        // assumption: the last cell is the legend
        table.cells.allElementsBoundByIndex.dropLast().last!
    }

    private var customTextField: XCUIElement {
        customCell.textFields.firstMatch
    }

    @discardableResult func tapAutomaticPortCell() -> Self {
        portCell(0).tap()
        return self
    }

    @discardableResult func tapPort51820Cell() -> Self {
        portCell(1).tap()
        return self
    }

    @discardableResult func tapPort53Cell() -> Self {
        portCell(2).tap()
        return self
    }

    @discardableResult func tapPortCustomCell() -> Self {
        customCell.tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.navigationBars.buttons.element(boundBy: 0).tap()
        return self
    }

    @discardableResult func typeTextIntoCustomField(_ text: String) -> Self {
        customTextField.tapWhenHittable()
        customTextField.typeText(text)
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
}
