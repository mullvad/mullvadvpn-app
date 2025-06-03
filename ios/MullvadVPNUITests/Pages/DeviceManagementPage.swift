//
//  DeviceManagementPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

/// Page class for the "too many devices" page shown when logging on to an account with too many devices
class DeviceManagementPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app
            .descendants(matching: .any)
            .matching(
                identifier: AccessibilityIdentifier.deviceManagementView.asString
            ).element
        waitForPageToBeShown()
    }

    @discardableResult func waitForNoLoading() -> Self {
        XCTAssertTrue(
            app.otherElements[.deviceRemovalProgressView]
                .waitForNonExistence(timeout: BaseUITestCase.longTimeout)
        )

        return self
    }

    @discardableResult func waitForDeviceList() -> Self {
        XCTAssertTrue(
            app
                .collectionViews[AccessibilityIdentifier.deviceListView]
                .waitForExistence(timeout: BaseUITestCase.longTimeout)
        )

        return self
    }

    @discardableResult func tapRemoveDeviceButton(cellIndex: Int) -> Self {
        app
            .cells.element(boundBy: cellIndex)
            .buttons[AccessibilityIdentifier.deviceCellRemoveButton]
            .tap()

        return self
    }

    @discardableResult func tapContinueWithLoginButton() -> Self {
        app.buttons[AccessibilityIdentifier.continueWithLoginButton].tap()
        return self
    }

    @discardableResult public func verifyCurrentDeviceExists() -> Self {
        XCTAssertTrue(
            app.staticTexts["Current device"]
                .waitForExistence(timeout: BaseUITestCase.defaultTimeout)
        )

        return self
    }

    @discardableResult public func verifyNoDeviceCanBeRemoved() -> Self {
        XCTAssertTrue(
            app
                .buttons[AccessibilityIdentifier.deviceCellRemoveButton]
                .waitForNonExistence(timeout: BaseUITestCase.defaultTimeout)
        )

        return self
    }

    @discardableResult public func verifyRemovableDeviceCount(_ expectedCount: Int) -> Self {
        XCTAssertEqual(
            app.buttons.matching(
                identifier: AccessibilityIdentifier.deviceCellRemoveButton.asString
            )
            .count,

            expectedCount
        )
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
