//
//  AddAccessMethodPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class AddAccessMethodPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.tables[.addAccessMethodTableView]
        waitForPageToBeShown()
    }

    @discardableResult func tapNameCell() -> Self {
        app.cells[AccessibilityIdentifier.accessMethodNameTextField]
            .tap()
        return self
    }

    @discardableResult func tapTypeCell() -> Self {
        app.cells[AccessibilityIdentifier.accessMethodProtocolSelectionCell]
            .tap()
        return self
    }

    @discardableResult func tapShadowsocksTypeValueCell() -> Self {
        app.tables[AccessibilityIdentifier.accessMethodProtocolPickerView].staticTexts["Shadowsocks"].tap()
        return self
    }

    @discardableResult func tapSOCKS5TypeValueCell() -> Self {
        app.tables[AccessibilityIdentifier.accessMethodProtocolPickerView].staticTexts["SOCKS5"].tap()
        return self
    }

    @discardableResult func tapServerCell() -> Self {
        app.cells[AccessibilityIdentifier.socks5ServerCell]
            .tap()
        return self
    }

    @discardableResult func tapPortCell() -> Self {
        app.cells[AccessibilityIdentifier.socks5PortCell]
            .tap()
        return self
    }

    @discardableResult func tapAuthenticationSwitch() -> Self {
        app.switches[AccessibilityIdentifier.socks5AuthenticationSwitch]
            .tap()
        return self
    }

    @discardableResult func tapAddButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodAddButton]
            .tap()
        return self
    }

    @discardableResult func waitForAPIUnreachableLabel() -> Self {
        XCTAssertTrue(
            app.staticTexts[AccessibilityIdentifier.addAccessMethodTestStatusUnreachableLabel]
                .existsAfterWait(timeout: .long)
        )
        return self
    }
}

class AddAccessMethodAPIUnreachableAlert: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.otherElements[.accessMethodUnreachableAlert]
        waitForPageToBeShown()
    }

    @discardableResult func tapSaveButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodUnreachableSaveButton].tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodUnreachableBackButton].tap()
        return self
    }
}
