//
//  SafariApp.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-05-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class SafariApp {
    let app = XCUIApplication(bundleIdentifier: "com.apple.mobilesafari")

    func launch() {
        app.launch()
    }

    @discardableResult func tapAddressBar() -> Self {
        app.textFields.firstMatch.tap()
        return self
    }

    @discardableResult func enterText(_ text: String) -> Self {
        app.typeText(text)
        return self
    }
}
