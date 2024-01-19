//
//  ProblemReportPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class ProblemReportPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        pageAccessibilityIdentifier = .problemReportView

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
        app.otherElements[AccessibilityIdentifier.problemReportAppLogsButton]
            .tap()

        return self
    }

    @discardableResult func tapSendButton() -> Self {
        app.otherElements[AccessibilityIdentifier.problemReportSendButton]
            .tap()

        return self
    }
}
