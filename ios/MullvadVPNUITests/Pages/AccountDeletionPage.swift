// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

class AccountDeletionPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.deleteAccountView]
        waitForPageToBeShown()
    }

    @discardableResult func tapTextField() -> Self {
        app.textFields[AccessibilityIdentifier.deleteAccountTextField].tap()
        return self
    }

    @discardableResult func tapDeleteAccountButton() -> Self {
        app.otherElements[.deleteAccountView].buttons[AccessibilityIdentifier.deleteButton].tap()
        return self
    }

    @discardableResult func tapCancelButton() -> Self {
        app.buttons[AccessibilityIdentifier.cancelButton].tap()
        return self
    }
}
