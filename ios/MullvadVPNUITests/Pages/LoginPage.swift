// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
        let submitButtonExist = app.buttons[AccessibilityIdentifier.loginButton]
            .existsAfterWait()
        XCTAssertTrue(submitButtonExist, "Account number submit button shown")
        return self
    }

    @discardableResult public func tapAccountNumberSubmitButton() -> Self {
        app.buttons[AccessibilityIdentifier.loginButton].tap()
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
        let elementExists = elementQuery.firstMatch.existsAfterWait(
            timeout: .longerThanMullvadAPITimeout
        )
        XCTAssertTrue(elementExists, "Fail icon shown")
        return self
    }

    /// Checks whether success icon is being shown
    func getSuccessIconShown() -> Bool {
        app.images[.statusImageView].existsAfterWait(timeout: .longerThanMullvadAPITimeout)
    }
}
