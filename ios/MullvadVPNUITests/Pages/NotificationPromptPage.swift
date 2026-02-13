//
//  NotificationPromptPage.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-02-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import XCTest

class NotificationPromptPage: Page {

    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.notificationPromptView]
        waitForPageToBeShown()
    }

    @discardableResult func tapSkipButton() -> Self {
        let button = app.buttons[AccessibilityIdentifier.notificationPromptSkipButton]
        if button.existsAfterWait() {
            button.tap()
        }
        return self
    }
}
