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

class APIAccessPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.otherElements[.apiAccessView]
        waitForPageToBeShown()
    }

    @discardableResult func tapAddButton() -> Self {
        app.buttons[AccessibilityIdentifier.addAccessMethodButton]
            .tap()
        return self
    }

    func getAccessMethodCells() -> [XCUIElement] {
        var elements = app.collectionViews[AccessibilityIdentifier.apiAccessListView].buttons.allElementsBoundByIndex
        elements.removeFirst()
        elements.removeLast()
        return elements
    }

    func getAccessMethodCell(accessibilityId: AccessibilityIdentifier) -> XCUIElement {
        app.buttons[accessibilityId]
    }

    func editAccessMethod(_ named: String) {
        app.buttons[named].tap()
    }
}
