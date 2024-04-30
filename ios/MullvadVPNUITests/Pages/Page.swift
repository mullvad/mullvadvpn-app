//
//  Page.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class Page {
    let app: XCUIApplication
    var pageAccessibilityIdentifier: AccessibilityIdentifier?

    @discardableResult init(_ app: XCUIApplication) {
        self.app = app
    }

    func waitForPageToBeShown() {
        if let pageAccessibilityIdentifier = self.pageAccessibilityIdentifier {
            XCTAssert(
                self.app.descendants(matching: .any).matching(identifier: pageAccessibilityIdentifier.rawValue)
                    .firstMatch
                    .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
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
        app.toolbars.buttons["Done"].tap()
        return self
    }

    @discardableResult func tapWhereStatusBarShouldBeToScrollToTopMostPosition() -> Self {
        app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0)).tap()
        return self
    }
}
