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

class EditCustomListLocationsPage: Page {
    enum Action {
        case add, edit
    }

    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.editCustomListEditLocationsView]
        waitForPageToBeShown()
    }

    @discardableResult func scrollToLocationWith(identifier: String) -> Self {
        let tableView = app.tables[.editCustomListEditLocationsTableView]
        tableView.cells[identifier].tap()
        return self
    }

    @discardableResult func toggleLocationCheckmarkWith(identifier: String) -> Self {
        let locationCell = app.tables[.editCustomListEditLocationsTableView].cells[identifier]
        locationCell.buttons[.customListLocationCheckmarkButton].tap()
        return self
    }

    @discardableResult func unfoldLocationwith(identifier: String) -> Self {
        let locationCell = app.tables[.editCustomListEditLocationsTableView].cells[identifier]
        let expandCellButton = locationCell.buttons["expandButton"]
        if expandCellButton.exists {
            expandCellButton.tap()
        }
        return self
    }

    @discardableResult func collapseLocationwith(identifier: String) -> Self {
        let locationCell = app.tables[.editCustomListEditLocationsTableView].cells[identifier]
        let collapseCellButton = locationCell.buttons["collapseButton"]
        if collapseCellButton.exists {
            collapseCellButton.tap()
        }
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        app.navigationBars["Locations"].buttons.firstMatch.tap()
        return self
    }
}
