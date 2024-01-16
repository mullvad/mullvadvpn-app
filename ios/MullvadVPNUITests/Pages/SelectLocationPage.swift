//
//  SelectLocationPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class SelectLocationPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .selectLocationView
    }

    @discardableResult func tapLocationCell(withName name: String) -> Self {
        app.tables[AccessibilityIdentifier.selectLocationTableView.rawValue].cells.staticTexts[name].tap()
        return self
    }

    @discardableResult func tapLocationCellExpandButton(withName name: String) -> Self {
        let table = app.tables[AccessibilityIdentifier.selectLocationTableView.rawValue]
        let matchingCells = table.cells.containing(.any, identifier: name)
        let buttons = matchingCells.buttons
        let expandButton = buttons[AccessibilityIdentifier.collapseButton.rawValue]

        expandButton.tap()

        return self
    }
}
