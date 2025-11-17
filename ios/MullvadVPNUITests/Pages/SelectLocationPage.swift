//
//  SelectLocationPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class SelectLocationPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.collectionViews[.selectLocationView]
        waitForPageToBeShown()
    }

    @discardableResult func tapLocationCell(withName name: String) -> Self {
        app.buttons[AccessibilityIdentifier.locationListItem(name)]
            .tap()
        return self
    }

    @discardableResult func tapLocationCellExpandButton(withName name: String) -> Self {
        let cell = app.buttons[AccessibilityIdentifier.locationListItem(name)]
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapAddNewCustomList() -> Self {
        let addNewCustomListButton = app.buttons[AccessibilityIdentifier.addNewCustomListButton]
        addNewCustomListButton.tap()
        return self
    }

    @discardableResult func editExistingCustomLists() -> Self {
        let editCustomListsButton = app.buttons[AccessibilityIdentifier.editCustomListButton]
        editCustomListsButton.tap()
        return self
    }

    @discardableResult func cellWithIdentifier(identifier: AccessibilityIdentifier) -> XCUIElement {
        app.buttons[identifier]
    }

    @discardableResult func tapFilterButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationFilterButton]
            .firstMatch
            .tap()
        return self
    }

    @discardableResult func tapMenuButton() -> Self {
        app.images[AccessibilityIdentifier.selectLocationToolbarMenu].tap()
        return self
    }

    @discardableResult func tapDoneButton() -> Self {
        app.buttons[AccessibilityIdentifier.closeSelectLocationButton].tap()
        return self
    }

    func locationCellIsExpanded(_ name: String) -> Bool {
        let matchingCells = app.cells.containing(.any, identifier: name)
        return matchingCells.buttons[AccessibilityIdentifier.expandButton].exists ? false : true
    }

    func verifyEditCustomListsButtonIs(enabled: Bool) {
        let editCustomListsButton = app.buttons[AccessibilityIdentifier.editCustomListButton]
        XCTAssertTrue(editCustomListsButton.isEnabled == enabled)
    }
}
