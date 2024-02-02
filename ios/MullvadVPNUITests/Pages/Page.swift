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

    public func waitForPageToBeShown() {
        if let pageAccessibilityIdentifier = self.pageAccessibilityIdentifier {
            XCTAssert(
                self.app.otherElements[pageAccessibilityIdentifier]
                    .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
            )
        }
    }

    @discardableResult public func enterText(_ text: String) -> Self {
        app.typeText(text)
        return self
    }

    /// Fast swipe down action to dismiss a modal view. Will swipe on the middle of the screen.
    @discardableResult func swipeDownToDismissModal() -> Self {
        app.swipeDown(velocity: .fast)
        return self
    }
}
