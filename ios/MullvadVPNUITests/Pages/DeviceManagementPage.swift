//
//  DeviceManagementPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-27.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

/// Page class for the "too many devices" page shown when logging on to an account with too many devices
class DeviceManagementPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.deviceManagementView]
        waitForPageToBeShown()
    }

    @discardableResult func tapRemoveDeviceButton(cellIndex: Int) -> Self {
        app
            .otherElements.matching(identifier: AccessibilityIdentifier.deviceCell.asString).element(boundBy: cellIndex)
            .buttons[AccessibilityIdentifier.deviceCellRemoveButton]
            .tap()

        return self
    }

    @discardableResult func tapContinueWithLoginButton() -> Self {
        app.buttons[AccessibilityIdentifier.continueWithLoginButton].tap()
        return self
    }
}

/// Confirmation alert displayed when removing a device
class DeviceManagementLogOutDeviceConfirmationAlert: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.otherElements[.alertContainerView]
        waitForPageToBeShown()
    }

    @discardableResult func tapYesLogOutDeviceButton() -> Self {
        app.buttons[AccessibilityIdentifier.logOutDeviceConfirmButton]
            .tap()
        return self
    }
}
