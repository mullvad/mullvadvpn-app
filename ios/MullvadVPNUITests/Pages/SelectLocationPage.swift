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

        self.pageElement = app.scrollViews[.selectLocationView]
        waitForPageToBeShown()
    }

    @discardableResult func tapLocationCell(withName name: String) -> Self {
        app
            .otherElements[AccessibilityIdentifier.locationListItem(name)]
            .buttons[AccessibilityIdentifier.locationItem]
            .tap()
        return self
    }

    @discardableResult func tapCountryLocationCellExpandButton(withName name: String) -> Self {
        let cell = app.otherElements[AccessibilityIdentifier.locationListItem(name)]
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapCityLocationCellExpandButton(withIndex: Int) -> Self {
        let cell = app.otherElements[AccessibilityIdentifier.locationChildren].firstMatch.otherElements
            .element(boundBy: withIndex)
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapRelayLocationCell(withIndex: Int) -> Self {
        let cell = app.otherElements[AccessibilityIdentifier.locationChildren]
            .firstMatch
            .otherElements[AccessibilityIdentifier.locationChildren]
            .firstMatch
            .buttons
            .element(boundBy: withIndex)
        cell.tap()
        return self
    }

    @discardableResult func tapLocationCellExpandButton(withName name: String) -> Self {
        let cell = app.otherElements[AccessibilityIdentifier.locationListItem(name)]
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapLocationCellCollapseButton(withName name: String) -> Self {
        let table = app.tables[AccessibilityIdentifier.selectLocationTableView]
        let matchingCells = table.cells.containing(.any, identifier: name)
        let buttons = matchingCells.buttons
        let collapseButton = buttons[AccessibilityIdentifier.collapseButton]

        collapseButton.tap()

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
        // Custom lists, countries and citys are otherElements, relays are buttons.
        app.otherElements[identifier].exists ? app.otherElements[identifier] : app.buttons[identifier]
    }

    @discardableResult func tapFilterButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationFilterButton].tap()
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
