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
        // Activate the text field
        editCustomListNameCell.tap()
        // Select the entire text with a triple tap
        editCustomListNameCell.tap(withNumberOfTaps: 3, numberOfTouches: 1)
        // Tap the "delete" key on the on-screen keyboard, the case is sensitive.
        // However, on a simulator the keyboard isn't visible by default, so we
        // need to take that into consideration.
        if app.keys["delete"].isHittable {
            app.keys["delete"].tap()
        }
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
