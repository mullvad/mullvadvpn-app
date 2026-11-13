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

class AddAccessMethodPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.scrollViews[.addAccessMethodScrollView]
        waitForPageToBeShown()
    }

    @discardableResult func tapNameCell() -> Self {
        tapInputField(accessibilityIdentifier: .accessMethodNameTextField)
        return self
    }

    @discardableResult func tapTypeCell() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodProtocolSelectionCell]
            .tap()
        return self
    }

    @discardableResult func tapShadowsocksTypeValueCell() -> Self {
        app.collectionViews[AccessibilityIdentifier.accessMethodProtocolPickerView].staticTexts["Shadowsocks"].tap()
        return self
    }

    @discardableResult func tapSOCKS5TypeValueCell() -> Self {
        app.collectionViews[AccessibilityIdentifier.accessMethodProtocolPickerView].staticTexts["SOCKS5"].tap()
        return self
    }

    @discardableResult func tapServerCell() -> Self {
        tapInputField(accessibilityIdentifier: .addAccessMethodServerCell)
        return self
    }

    @discardableResult func tapPortCell() -> Self {
        tapInputField(accessibilityIdentifier: .addAccessMethodPortCell)
        return self
    }

    @discardableResult func tapAuthenticationSwitch() -> Self {
        app.switches[.socks5AuthenticationSwitch].switches.firstMatch.tap()
        return self
    }

    @discardableResult func tapAddButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodAddButton]
            .tap()
        return self
    }

    private func tapInputField(accessibilityIdentifier: AccessibilityIdentifier) {
        let view = app.otherElements[accessibilityIdentifier]
        let firstTextField = view.textFields.firstMatch
        XCTAssertTrue(firstTextField.existsAfterWait())
        firstTextField.tap()
    }
}

class AddAccessMethodAPIUnreachableAlert: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.otherElements[.alertContainerView]
        waitForPageToBeShown()
    }

    @discardableResult func tapSaveButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodUnreachableSaveButton].tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodUnreachableCancelButton].tap()
        return self
    }
}
