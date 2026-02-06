//
//  APIAccessPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
