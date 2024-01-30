//
//  OutOfTimePage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-20.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class OutOfTimePage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .outOfTimeView
        waitForPageToBeShown()
    }
}
