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

/// Generic alert "page"
class Alert: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.alertContainerView]
        waitForPageToBeShown()
    }

    @discardableResult func tapOkay() -> Self {
        app.buttons[AccessibilityIdentifier.alertOkButton].tap()
        return self
    }
}
