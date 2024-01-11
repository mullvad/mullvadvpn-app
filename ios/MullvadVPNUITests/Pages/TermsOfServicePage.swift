//
//  TermsOfServicePage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class TermsOfServicePage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .termsOfServiceView
    }

    @discardableResult func tapAgree() -> Self {
        app.buttons[AccessibilityIdentifier.agreeButton.rawValue].tap()
        return self
    }
}
