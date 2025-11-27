//
//  LoginPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class LoginPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.loginView]
        waitForPageToBeShown()
    }

    @discardableResult public func tapAccountNumberTextField() -> Self {
        app.textFields[AccessibilityIdentifier.loginTextField].tap()
        return self
    }

    @discardableResult public func waitForAccountNumberSubmitButton() -> Self {
        let submitButtonExist = app.buttons[AccessibilityIdentifier.loginTextFieldButton]
            .existsAfterWait()
        XCTAssertTrue(submitButtonExist, "Account number submit button shown")
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

    @discardableResult public func verifySuccessIconShown() -> Self {
        let isShown = getSuccessIconShown()

        XCTAssertTrue(isShown, "Success icon shown")

        return self
    }

    @discardableResult public func confirmAccountCreation() -> Self {
        app.buttons[AccessibilityIdentifier.createAccountConfirmationButton].tap()
        return self
    }

    @discardableResult public func tryConfirmAccountCreation() -> Self {
        if app.buttons[AccessibilityIdentifier.createAccountConfirmationButton].existsAfterWait(timeout: .short) {
            return confirmAccountCreation()
        }
        return self
    }

    @discardableResult public func verifyFailIconShown() -> Self {
        let predicate = NSPredicate(format: "identifier == 'statusImageView' AND value == 'fail'")
        let elementQuery = app.images.containing(predicate)
        let elementExists = elementQuery.firstMatch.existsAfterWait(timeout: .veryLong)
        XCTAssertTrue(elementExists, "Fail icon shown")
        return self
    }

    /// Checks whether success icon is being shown
    func getSuccessIconShown() -> Bool {
        app.images[.statusImageView].existsAfterWait(timeout: .long)
    }
}
