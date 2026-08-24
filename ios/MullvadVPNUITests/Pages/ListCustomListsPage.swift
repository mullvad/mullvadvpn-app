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
