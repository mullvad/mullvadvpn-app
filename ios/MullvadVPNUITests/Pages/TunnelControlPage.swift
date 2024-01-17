//
//  TunnelControlPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class TunnelControlPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .tunnelControlView
    }

    @discardableResult func tapSelectLocationButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationButton.rawValue].tap()
        return self
    }

    @discardableResult func tapSettingsButton() -> Self {
        app.buttons[AccessibilityIdentifier.settingsButton.rawValue].tap()
        return self
    }

}
