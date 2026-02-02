//
//  LwoObfuscationSettingsPage.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class LwoObfuscationSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    private var table: XCUIElement {
        app.collectionViews[AccessibilityIdentifier.wireGuardObfuscationLwoTable]
    }

    private func portCell(_ index: Int) -> XCUIElement {
        table.cells.element(boundBy: index)
    }

    private var customCell: XCUIElement {
        // assumption: the last cell is the legend
        table.cells.allElementsBoundByIndex.dropLast().last!
    }

    private var customTextField: XCUIElement {
        customCell.textFields.firstMatch
    }

    @discardableResult func tapAutomaticPortCell() -> Self {
        portCell(0).tap()
        return self
    }

    @discardableResult func tapCustomCell() -> Self {
        customCell.tap()
        return self
    }

    @discardableResult func typeTextIntoCustomField(_ text: String) -> Self {
        customTextField.typeText(text)
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.navigationBars.buttons.element(boundBy: 0).tap()
        return self
    }
}
