//
//  CustomListPage.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class CustomListPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .newCustomListView
        waitForPageToBeShown()
    }

    @discardableResult func verifyCreateButtonIs(enabled: Bool) -> Self {
        let saveOrCreateButton = app.buttons[.saveCreateCustomListButton]
        XCTAssertTrue(saveOrCreateButton.isEnabled == enabled)
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

    @discardableResult func editCustomList(name: String) -> Self {
        let editCustomListNameCell = app.cells[.customListEditNameFieldCell]
        editCustomListNameCell.tap()
        editCustomListNameCell.typeText(name)
        return self
    }

    @discardableResult func deleteCustomList(named customListName: String) -> Self {
        let deleteCustomListCell = app.cells[.customListEditDeleteListCell]
        deleteCustomListCell.tap()
        app.buttons[.confirmDeleteCustomListButton].tap()
        return self
    }

    @discardableResult func addOrEditLocations() -> Self {
        app.cells[.customListEditAddOrEditLocationCell].tap()
        return self
    }
}
