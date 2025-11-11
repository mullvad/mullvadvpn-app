//
//  SetUpAccountCompletedPage.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-11-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class SetUpAccountCompletedPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.setUpAccountCompletedView]
        waitForPageToBeShown()
    }
}
