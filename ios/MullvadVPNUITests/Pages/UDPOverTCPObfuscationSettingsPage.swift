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

class UDPOverTCPObfuscationSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    private var table: XCUIElement {
        app.collectionViews[AccessibilityIdentifier.wireGuardObfuscationUdpOverTcpTable]
    }

    private func portCell(_ index: Int) -> XCUIElement {
        table.cells.element(boundBy: index)
    }

    @discardableResult func tapAutomaticPortCell() -> Self {
        portCell(0).tap()
        return self
    }

    @discardableResult func tapPort80Cell() -> Self {
        portCell(1).tap()
        return self
    }

    @discardableResult func tapPort443Cell() -> Self {
        portCell(2).tap()
        return self
    }

    @discardableResult func tapPort5001Cell() -> Self {
        portCell(3).tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.navigationBars.buttons.element(boundBy: 0).tap()
        return self
    }

    @discardableResult func verifyUDPOverTCPPort80Selected() -> Self {
        let cells = table.cells
        var isCorrectPortSelected = false
        for i in 0..<cells.count {
            let cell = cells.element(boundBy: i)

            if cell.images[AccessibilityIdentifier.selectedSingleOption].exists {
                if cell.staticTexts["80"].exists {
                    isCorrectPortSelected = true
                }
            }
        }
        XCTAssertTrue(isCorrectPortSelected, "Port 80 is not selected")
        return self
    }
}
