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
        app.textFields[AccessibilityIdentifier.loginTextField].tap()
        return self
    }

    @discardableResult public func tapAccountNumberSubmitButton() -> Self {
        app.buttons[AccessibilityIdentifier.loginTextFieldButton].tap()
        return self
    }

    @discardableResult public func tapCreateAccountButton() -> Self {
        app.buttons[AccessibilityIdentifier.createAccountButton].tap()
        return self
    }

    @discardableResult public func verifyDeviceLabelShown() -> Self {
        XCTAssertTrue(
            app.staticTexts[AccessibilityIdentifier.headerDeviceNameLabel]
                .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
        )

        return self
    }

    @discardableResult public func verifySuccessIconShown() -> Self {
        _ = app.images.element(matching: .image, identifier: "IconSuccess")
            .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
        return self
    }

    @discardableResult public func verifyFailIconShown() -> Self {
        _ = app.images.element(matching: .image, identifier: "IconFail")
            .waitForExistence(timeout: BaseUITestCase.longTimeout)
        return self
    }
}
