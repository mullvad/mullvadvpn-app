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
        self.pageElement = app.otherElements[.apiAccessView]
        waitForPageToBeShown()
    }

    @discardableResult func tapAddButton() -> Self {
        app.buttons[AccessibilityIdentifier.addAccessMethodButton]
            .tap()
        return self
    }

    func getAccessMethodCells() -> [XCUIElement] {
        app.otherElements[AccessibilityIdentifier.apiAccessView].cells.allElementsBoundByIndex
    }

    func getAccessMethodCell(accessibilityId: AccessibilityIdentifier) -> XCUIElement {
        app.otherElements[AccessibilityIdentifier.apiAccessView].cells[accessibilityId]
    }

    func getMethodIsInUse(_ cell: XCUIElement) -> Bool {
        cell.staticTexts.allElementsBoundByIndex.contains { element in
            element.label == "In use"
        }
    }

    func getMethodIsDisabled(_ cell: XCUIElement) -> Bool {
        cell.staticTexts.allElementsBoundByIndex.contains { element in
            element.label == "Disabled"
        }
    }
}
