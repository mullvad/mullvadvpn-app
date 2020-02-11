//
//  MullvadVPNScreenshots.swift
//  MullvadVPNScreenshots
//
//  Created by pronebird on 04/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

class MullvadVPNScreenshots: XCTestCase {

    override func setUp() {
        // Put setup code here. This method is called before the invocation of each test method in the class.

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false

        // In UI tests it’s important to set the initial state - such as interface orientation - required for your tests before they run. The setUp method is a good place to do this.
    }

    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testTakeScreenshots() {
        let accountToken = Bundle(for: Self.self).infoDictionary?["MullvadAccountToken"] as! String

        // UI tests must launch the application that they test.
        let app = XCUIApplication()
        setupSnapshot(app)

        app.launch()

        snapshot("Login")

        // Enter account token
        let textField = app.textFields["LoginTextField"]
        textField.tap()
        textField.typeText(accountToken)

        // Tap "Log in" button to log in
        app.toolbars["Toolbar"].buttons["LoginBarButtonItem"].tap()

        // Connect VPN
        _ = app.buttons["ConnectButton"].waitForExistence(timeout: 10)
        app.buttons["ConnectButton"].tap()

        // Wait for Disconnect button to appear
        _ = app.buttons["DisconnectButton"].waitForExistence(timeout: 2)

        snapshot("MainSecured")

        // Open Settings
        app.buttons["SettingsButton"].tap()

        // Tap on WireGuard key cell
        _ = app.tables.cells["WireGuardKeyCell"].waitForExistence(timeout: 2)
        app.tables.cells["WireGuardKeyCell"].tap()

        snapshot("WireGuardKeys")

        // Tap back button
        app.navigationBars.buttons.firstMatch.tap()

        // Tap "Account" cell
        _ = app.tables.cells["AccountCell"].waitForExistence(timeout: 2)
        app.tables.cells["AccountCell"].tap()

        // Hit "Log out" button
        _ = app.buttons["LogoutButton"].waitForExistence(timeout: 2)
        app.buttons["LogoutButton"].tap()

        // Wait for Login view to appear after log out
        _ = app.textFields["LoginTextField"].waitForExistence(timeout: 10)
    }
}
