// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
