//
//  ListCustomListsPage.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class ListCustomListsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.listCustomListsView]
        waitForPageToBeShown()
    }

    /// This function taps on a given custom list in the Edit Custom List page.
    ///
    /// This functions assumes that all the custom lists are visible on a single page
    /// No scrolling will be attempted to scroll to find a custom list
    /// - Parameter customListName: The custom list to edit
    @discardableResult func selectCustomListToEdit(named customListName: String) -> Self {
        app.tables[.listCustomListsTableView].staticTexts[customListName].tap()
        return self
    }

    @discardableResult func tapDoneButton() -> Self {
        app.buttons[.listCustomListDoneButton].tap()
        return self
    }
}
