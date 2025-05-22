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
            .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
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

    @discardableResult public func verifyFailIconShown() -> Self {
        let predicate = NSPredicate(format: "identifier == 'statusImageView' AND value == 'fail'")
        let elementQuery = app.images.containing(predicate)
        let elementExists = elementQuery.firstMatch.waitForExistence(timeout: BaseUITestCase.longTimeout)
        XCTAssertTrue(elementExists, "Fail icon shown")
        return self
    }

    /// Checks whether success icon is being shown
    func getSuccessIconShown() -> Bool {
        // Success icon is only shown very briefly, since another view is presented after success icon is shown.
        // Therefore we need to poll faster than waitForElement function.
        let successIconDisplayedExpectation = XCTestExpectation(description: "Success icon shown")
        nonisolated(unsafe) var isShown = false
        let timer = Timer.scheduledTimer(withTimeInterval: 0.2, repeats: true) { [self] _ in
            DispatchQueue.main.async {
                let statusImageView = self.app.images[.statusImageView]

                if statusImageView.exists {
                    if statusImageView.value as? String == "success" {
                        isShown = true
                        successIconDisplayedExpectation.fulfill()
                    }
                }
            }
        }

        _ = XCTWaiter.wait(for: [successIconDisplayedExpectation], timeout: BaseUITestCase.longTimeout)
        timer.invalidate()

        return isShown
    }
}
