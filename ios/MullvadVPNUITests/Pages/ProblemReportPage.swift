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

class ProblemReportPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.problemReportView]
        waitForPageToBeShown()
    }

    @discardableResult func tapEmailTextField() -> Self {
        app.textFields[AccessibilityIdentifier.problemReportEmailTextField]
            .tap()

        return self
    }

    @discardableResult func tapMessageTextView() -> Self {
        app.textViews[AccessibilityIdentifier.problemReportMessageTextView]
            .tap()

        return self
    }

    @discardableResult func tapViewAppLogsButton() -> Self {
        app.buttons[AccessibilityIdentifier.problemReportAppLogsButton]
            .tap()

        return self
    }

    @discardableResult func tapSendButton() -> Self {
        app.buttons[AccessibilityIdentifier.problemReportSendButton]
            .tap()

        return self
    }
}
