//
//  EditCustomListLocationsPage.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
