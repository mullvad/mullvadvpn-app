//
//  LoginPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class LoginPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .loginView
        waitForPageToBeShown()
    }

    @discardableResult public func tapAccountNumberTextField() -> Self {
        app.textFields[AccessibilityIdentifier.loginTextField.rawValue].tap()
        return self
    }

    @discardableResult public func tapAccountNumberSubmitButton() -> Self {
        app.buttons[AccessibilityIdentifier.loginTextFieldButton.rawValue].tap()
        return self
    }

    @discardableResult public func verifyDeviceLabelShown() -> Self {
        XCTAssert(app.staticTexts[AccessibilityIdentifier.headerDeviceNameLabel.rawValue].waitForExistence(timeout: 10))
        return self
    }

    @discardableResult public func verifySuccessIconShown() -> Self {
        app.images.element(matching: .image, identifier: "IconSuccess")
        return self
    }

    @discardableResult public func verifyFailIconShown() -> Self {
        app.images.element(matching: .image, identifier: "IconFail")
        return self
    }
}
