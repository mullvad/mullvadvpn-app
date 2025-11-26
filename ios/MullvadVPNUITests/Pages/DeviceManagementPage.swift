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

        self.pageElement =
            app
            .descendants(matching: .any)
            .matching(
                identifier: AccessibilityIdentifier.deviceManagementView.asString
            ).element
        waitForPageToBeShown()
    }

    @discardableResult func waitForNoLoading() -> Self {
        XCTAssertTrue(
            app.otherElements[.deviceRemovalProgressView]
                .notExistsAfterWait(timeout: .long)
        )

        return self
    }

    @discardableResult func waitForDeviceList() -> Self {
        XCTAssertTrue(
            app
                .collectionViews[AccessibilityIdentifier.deviceManagementView]
                .existsAfterWait(timeout: .long)
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
                .existsAfterWait()
        )

        return self
    }

    @discardableResult public func verifyCurrentDeviceCannotBeRemoved() -> Self {
        let cells = app.cells
        let buttons = cells.buttons

        // Button count should equal the amount of cells, except for the cell that cannot
        // be removed and the information text cell at the top of the page.
        XCTAssertEqual(buttons.count, cells.count - 2)

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
