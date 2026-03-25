//
//  MultihopPage.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import XCTest

class MultihopPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.multihopView]
        waitForPageToBeShown()
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func verifyOnePage() -> Self {
        XCTAssertEqual(app.pageIndicators.firstMatch.value as? String, "page 1 of 1")
        return self
    }

    @discardableResult func tapMultihopState(_ state: MultihopState) -> Self {
        app.buttons[AccessibilityIdentifier.multihopState(state.description)].tap()
        return self
    }
}
