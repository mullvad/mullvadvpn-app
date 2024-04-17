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
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .selectLocationView
        waitForPageToBeShown()
    }

    @discardableResult func tapLocationCell(withName name: String) -> Self {
        app.tables[AccessibilityIdentifier.selectLocationTableView].cells.staticTexts[name].tap()
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

    func locationCellIsExpanded(_ name: String) -> Bool {
        let matchingCells = app.cells.containing(.any, identifier: name)
        return matchingCells.buttons[AccessibilityIdentifier.expandButton].exists ? false : true
    }

    @discardableResult
    func openCustomListsActions() -> Self {
        scrollToCustomListsSection()
        let customListEllipsisButton = app.buttons[AccessibilityIdentifier.openCustomListsMenuButton]
        customListEllipsisButton.tap()
        return self
    }

    @discardableResult
    func scrollToCustomListsSection() -> Self {
        let selectLocationTableView = app.tables[AccessibilityIdentifier.selectLocationTableView]
        selectLocationTableView.swipeDown(velocity: XCUIGestureVelocity(floatLiteral: 9999))
        return self
    }

    func tapAddNewCustomList() {
        let addNewCustomListButton = app.buttons[AccessibilityIdentifier.addNewCustomListButton]
        addNewCustomListButton.tap()
    }

    func editExistingCustomLists() {
        let editCustomListsButton = app.buttons[AccessibilityIdentifier.editCustomListButton]
        editCustomListsButton.tap()
    }

    func verifyEditCustomListsButtonIs(enabled: Bool) {
        let editCustomListsButton = app.buttons[AccessibilityIdentifier.editCustomListButton]
        XCTAssertTrue(editCustomListsButton.isEnabled == enabled)
    }
}
