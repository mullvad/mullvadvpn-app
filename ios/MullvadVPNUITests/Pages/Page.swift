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

    init(_ app: XCUIApplication) {
        self.app = app
    }

    public func enterText(_ text: String) -> Self {
        app.typeText(text)
        return self
    }

    public func waitForPageToBeShown() {
        if let pageAccessibilityIdentifier = self.pageAccessibilityIdentifier {
            XCTAssert(self.app.otherElements[pageAccessibilityIdentifier.rawValue].waitForExistence(timeout: 10))
        }
    }
}
