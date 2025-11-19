//
//  Page.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
                pageElement.existsAfterWait(),
                "Page is shown"
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
        app.swipeDown(velocity: .fast)
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
