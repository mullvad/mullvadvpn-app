//
//  APIAccessPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class APIAccessPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageAccessibilityIdentifier = .apiAccessView
        waitForPageToBeShown()
    }

    @discardableResult func tapAddButton() -> Self {
        app.buttons[AccessibilityIdentifier.addAccessMethodButton]
            .tap()
        return self
    }

    func getAccessMethodCells() -> [XCUIElement] {
        return app.otherElements[AccessibilityIdentifier.apiAccessView].cells.allElementsBoundByIndex
    }
}
