//
//  ShadowsocksObfuscationSettingsPage.swift
//  MullvadVPNUITests
//
//  Created by Andrew Bulhak on 2024-12-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class ShadowsocksObfuscationSettingsPage: Page {
    
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        // is this right?
        self.pageElement = app.otherElements[.settingsContainerView]
        waitForPageToBeShown()
    }
}
