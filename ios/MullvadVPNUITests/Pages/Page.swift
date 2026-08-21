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

@MainActor
class Page {
    let app: XCUIApplication

    /// Element in the page used to verify that the page is currently being shown, usually accessibilityIdentifier of the view controller's main view
    var pageElement: XCUIElement?

    @discardableResult init(_ app: XCUIApplication) {
        self.app = app
    }

    func waitForPageToBeShown() {
        if let pageElement {
            XCTAssertTrue(
                pageElement.existsAfterWait(timeout: .extremelyLong),
                "Page is not shown"
            )
        }
    }

    @discardableResult func enterText(_ text: String) -> Self {
        app.typeText(text)
        return self
    }

    @discardableResult func dismissKeyboard() -> Self {
        self.enterText("\n")
        return self
    }

    /// Fast swipe down action to dismiss a modal view. Will swipe on the middle of the screen.
    @discardableResult func swipeDownToDismissModal() -> Self {
        let start = app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.2))
        let end = app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.8))
        start.press(forDuration: 0, thenDragTo: end)
        return self
    }

    @discardableResult func tapKeyboardDoneButton() -> Self {
        app.toolbars.buttons[NSLocalizedString("Done", comment: "")].tap()
        return self
    }

    @discardableResult func tapWhereStatusBarShouldBeToScrollToTopMostPosition() -> Self {
        // Tapping but not at center x coordinate because on iPad there's an ellipsis button in the center of the status bar
        app.coordinate(withNormalizedOffset: CGVector(dx: 0.75, dy: 0)).tap()
        return self
    }
}
