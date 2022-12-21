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
        // Put setup code here. This method is called before the invocation of
        // each test method in the class.

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false

        // In UI tests it’s important to set the initial state - such as interface orientation -
        // required for your tests before they run. The setUp method is a good place to do this.
    }

    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of
        // each test method in the class.
    }

    func testTakeScreenshots() {
        let accountToken = Bundle(for: Self.self).infoDictionary?["MullvadAccountToken"] as! String

        // UI tests must launch the application that they test.
        let app = XCUIApplication()
        setupSnapshot(app)

        app.launch()

        // Dismiss terms of service screen
        _ = app.buttons["AgreeButton"].waitForExistence(timeout: 10)
        app.buttons["AgreeButton"].tap()

        // Wait for Login screen
        let textField = app.textFields["LoginTextField"]
        _ = textField.waitForExistence(timeout: 5)

        // Enter account token
        textField.tap()
        textField.typeText(accountToken)

        // Tap "Log in" button to log in
        if case .phone = UIDevice.current.userInterfaceIdiom {
            app.toolbars["Toolbar"].buttons["LoginBarButtonItem"].tap()
        } else {
            textField.typeText("\n")
        }

        // Select Sweden, Gothenburg in Select location controller
        if case .phone = UIDevice.current.userInterfaceIdiom {
            _ = app.buttons["SelectLocationButton"].waitForExistence(timeout: 10)
            app.buttons["SelectLocationButton"].tap()
        }

        let countryCell = app.cells["se"]
        let cityCell = app.cells["se-got"]

        _ = countryCell.waitForExistence(timeout: 2)

        if cityCell.exists {
            cityCell.tap()
        } else {
            countryCell.buttons["CollapseButton"].tap()
            cityCell.tap()
        }

        // Wait for Disconnect button to appear
        _ = app.buttons["DisconnectButton"].waitForExistence(timeout: 2)

        snapshot("MainSecured")

        // Re-open Select location controller (iPhone only)
        if case .phone = UIDevice.current.userInterfaceIdiom {
            app.buttons["SelectLocationButton"].tap()
            cityCell.buttons["CollapseButton"].tap()
            snapshot("SelectLocation")

            // Tap the "Done" button to dismiss the "Select location" controller
            app.navigationBars.buttons.firstMatch.tap()
        }

        // Open Settings
        app.buttons["SettingsButton"].tap()

        // Tap on preferences cell
        _ = app.tables.cells["PreferencesCell"].waitForExistence(timeout: 2)
        app.tables.cells["PreferencesCell"].tap()

        app.tables.element
            .cells
            .matching(NSPredicate(format: "identifier BEGINSWITH %@", "mullvadDNS"))
            .switches
            .matching(NSPredicate(format: "value = %@", "0"))
            .allElementsBoundByAccessibilityElement
            .forEach { $0.tap() }
        snapshot("Preferences")

        // Tap back button
        app.navigationBars.buttons.firstMatch.tap()

        // Tap "Account" cell
        _ = app.tables.cells["AccountCell"].waitForExistence(timeout: 2)
        app.tables.cells["AccountCell"].tap()

        // Wait for StoreKit to fetch subscriptions
        _ = app.buttons["PurchaseButton"].waitForExistence(timeout: 2)

        wait(for: [
            expectation(
                for: NSPredicate(format: "isEnabled = YES"),
                evaluatedWith: app.buttons["PurchaseButton"]
            ),
        ], timeout: 10)
        snapshot("Account")

        // Hit "Log out" button
        _ = app.buttons["LogoutButton"].waitForExistence(timeout: 2)
        app.buttons["LogoutButton"].tap()
        app.alerts.buttons.allElementsBoundByIndex.last?.tap()

        // Wait for Login view to appear after log out
        _ = app.textFields["LoginTextField"].waitForExistence(timeout: 10)
    }
}
