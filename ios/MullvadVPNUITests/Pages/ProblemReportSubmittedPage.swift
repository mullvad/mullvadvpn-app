//
//  ProblemReportSubmittedPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class ProblemReportSubmittedPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        pageAccessibilityIdentifier = .problemReportSubmittedView
        waitForPageToBeShown()
    }
}
