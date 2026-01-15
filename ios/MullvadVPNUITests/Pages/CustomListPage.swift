//
//  CustomListPage.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class CustomListPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.newCustomListView]
        waitForPageToBeShown()
    }

    @discardableResult func verifyCreateButtonIs(enabled: Bool) -> Self {
        let saveOrCreateButton = app.buttons[.saveCreateCustomListButton]
        XCTAssertTrue(saveOrCreateButton.isEnabled == enabled, "Verify state of create button")
        return self
    }

    @discardableResult func tapCreateListButton() -> Self {
        let saveOrCreateButton = app.buttons[.saveCreateCustomListButton]
        saveOrCreateButton.tap()
        return self
    }

    // It's the same button, the difference is just for semantics
    @discardableResult func tapSaveListButton() -> Self {
        tapCreateListButton()
    }

    @discardableResult func renameCustomList(name: String) -> Self {
        let editCustomListNameCell = app.cells[.customListEditNameFieldCell]
        let textField = editCustomListNameCell.textFields.firstMatch
        textField.clearText(app: app)
        editCustomListNameCell.typeText(name)
        return self
    }

    @discardableResult func deleteCustomList(named customListName: String) -> Self {
        let deleteCustomListCell = app.cells[.customListEditDeleteListCell]
        deleteCustomListCell.tap()

        app.buttons[AccessibilityIdentifier.confirmDeleteCustomListButton].tapWhenHittable()

        return self
    }

    @discardableResult func addOrEditLocations() -> Self {
        app.cells[.customListEditAddOrEditLocationCell].tap()
        return self
    }
}
