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

        self.pageElement = app.otherElements[.selectLocationView]
        waitForPageToBeShown()
    }

    @discardableResult func tapLocationCell(withName name: String) -> Self {
        app.tables[AccessibilityIdentifier.selectLocationTableView].cells.staticTexts[name].tap()
        return self
    }

    @discardableResult func tapCountryLocationCellExpandButton(withName name: String) -> Self {
        let cell = app.cells.containing(.any, identifier: name)
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapCountryLocationCellExpandButton(withIndex: Int) -> Self {
        let cell = app.cells.containing(.any, identifier: AccessibilityIdentifier.countryLocationCell.asString)
            .element(boundBy: withIndex)
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapCityLocationCellExpandButton(withIndex: Int) -> Self {
        let cell = app.cells.containing(.any, identifier: AccessibilityIdentifier.cityLocationCell.asString)
            .element(boundBy: withIndex)
        let expandButton = cell.buttons[AccessibilityIdentifier.expandButton]
        expandButton.tap()
        return self
    }

    @discardableResult func tapRelayLocationCell(withIndex: Int) -> Self {
        let cell = app.cells.containing(.any, identifier: AccessibilityIdentifier.relayLocationCell.asString)
            .element(boundBy: withIndex)
        cell.tap()
        return self
    }

    @discardableResult func tapLocationCellExpandButton(withName name: String) -> Self {
        let table = app.tables[AccessibilityIdentifier.selectLocationTableView]
        let matchingCells = table.cells.containing(.any, identifier: name)
        let buttons = matchingCells.buttons
        let expandButton = buttons[AccessibilityIdentifier.expandButton]

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

    @discardableResult func tapCustomListEllipsisButton() -> Self {
        // This wait should not be needed, but is due to the issues we are having with the ellipsis button
        _ = app.buttons[.openCustomListsMenuButton].waitForExistence(timeout: BaseUITestCase.shortTimeout)

        let customListEllipsisButtons = app.buttons
            .matching(identifier: AccessibilityIdentifier.openCustomListsMenuButton.asString).allElementsBoundByIndex

        // This is a workaround for an issue we have with the ellipsis showing up multiple times in the accessibility hieararchy even though in the view hierarchy there is only one
        // Only the actually visual one is hittable, so only the visible button will be tapped
        for ellipsisButton in customListEllipsisButtons where ellipsisButton.isHittable {
            ellipsisButton.tap()
            return self
        }

        XCTFail("Found no hittable custom list ellipsis button")

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

    @discardableResult func cellWithIdentifier(identifier: String) -> XCUIElement {
        app.tables[AccessibilityIdentifier.selectLocationTableView].cells[identifier]
    }

    @discardableResult func tapFilterButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationFilterButton].tap()
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
