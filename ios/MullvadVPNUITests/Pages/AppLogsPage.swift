//
//  AppLogsPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-26.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
