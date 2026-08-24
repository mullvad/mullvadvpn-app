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

class AppLogsPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.otherElements[.appLogsView]
        waitForPageToBeShown()
    }

    @discardableResult func tapShareButton() -> Self {
        app.buttons[.appLogsShareButton].tap()
        return self
    }

    @discardableResult func tapDoneButton() -> Self {
        app.buttons[.appLogsDoneButton].tap()
        return self
    }

    func getAppLogText() -> String {
        guard let logText = app.textViews[.problemReportAppLogsTextView].value as? String else {
            XCTFail("Failed to extract app log text")
            return String()
        }

        return logText
    }
}
