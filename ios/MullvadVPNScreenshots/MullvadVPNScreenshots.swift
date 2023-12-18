//
//  MullvadVPNScreenshots.swift
//  MullvadVPNScreenshots
//
//  Created by pronebird on 04/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

class MullvadVPNScreenshots: XCTestCase {
    let app = XCUIApplication()

    override func setUp() {
        // Put setup code here. This method is called before the invocation of
        // each test method in the class.

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false

        // Disable animations to speed up tests. This argument is picked up in AppDelegate.didFinishLaunchingWithOptions.
        app.launchArguments = ["DisableAnimations"]

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
        setupSnapshot(app, waitForAnimations: false)
        app.launch()

        // Dismiss terms of service screen
        _ = app.buttons[AccessibilityIdentifier.agreeButton.rawValue].waitForExistence(timeout: 10)
        app.buttons[AccessibilityIdentifier.agreeButton.rawValue].tap()

        // Dismiss changelog screen
        _ = app.buttons[AccessibilityIdentifier.alertOkButton.rawValue].waitForExistence(timeout: 10)
        app.buttons[AccessibilityIdentifier.alertOkButton.rawValue].tap()

        // Wait for Login screen
        let textField = app.textFields[AccessibilityIdentifier.loginTextField.rawValue]
        _ = textField.waitForExistence(timeout: 5)

        // Enter account token
        textField.tap()
        textField.typeText(accountToken)

        // Tap "Log in" button to log in
        if case .phone = UIDevice.current.userInterfaceIdiom {
            app.toolbars["Toolbar"].buttons[AccessibilityIdentifier.loginBarButton.rawValue].tap()
        } else {
            textField.typeText("\n")
        }

        // Select Sweden, Gothenburg in Select location controller
        if case .phone = UIDevice.current.userInterfaceIdiom {
            _ = app.buttons[AccessibilityIdentifier.selectLocationButton.rawValue].waitForExistence(timeout: 10)
            app.buttons[AccessibilityIdentifier.selectLocationButton.rawValue].tap()
        }

        let countryCell = app.cells["se"]
        let cityCell = app.cells["se-got"]

        _ = countryCell.waitForExistence(timeout: 2)

        if cityCell.exists {
            cityCell.tap()
        } else {
            _ = countryCell.buttons[AccessibilityIdentifier.collapseButton.rawValue].waitForExistence(timeout: 5)
            countryCell.buttons[AccessibilityIdentifier.collapseButton.rawValue].tap()
            cityCell.tap()
        }

        // Wait for Disconnect button to appear
        _ = app.buttons[AccessibilityIdentifier.disconnectButton.rawValue].waitForExistence(timeout: 2)

        snapshot("MainSecured")

        // Re-open Select location controller (iPhone only)
        if case .phone = UIDevice.current.userInterfaceIdiom {
            app.buttons[AccessibilityIdentifier.selectLocationButton.rawValue].tap()
            cityCell.buttons[AccessibilityIdentifier.collapseButton.rawValue].tap()
            snapshot("SelectLocation")

            // Tap the "Filter" button and expand each relay filter
            app.navigationBars.buttons["Filter"].tap()
            app.otherElements["Ownership"].buttons[AccessibilityIdentifier.collapseButton.rawValue].tap()
            app.otherElements["Providers"].buttons[AccessibilityIdentifier.collapseButton.rawValue].tap()
            snapshot("RelayFilter")

            app.navigationBars.buttons["Cancel"].tap()
            app.navigationBars.buttons["Done"].tap()
        }

        // Open Settings
        app.buttons[AccessibilityIdentifier.settingsButton.rawValue].tap()

        // Tap on preferences cell
        _ = app.tables.cells[AccessibilityIdentifier.preferencesCell.rawValue].waitForExistence(timeout: 2)
        app.tables.cells[AccessibilityIdentifier.preferencesCell.rawValue].tap()

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

        // Tap dismiss button
        app.navigationBars.buttons.firstMatch.tap()

        // Open Account
        app.buttons[AccessibilityIdentifier.accountButton.rawValue].tap()

        // Wait for StoreKit to fetch subscriptions
        _ = app.buttons[AccessibilityIdentifier.purchaseButton.rawValue].waitForExistence(timeout: 2)

        wait(for: [
            expectation(
                for: NSPredicate(format: "isEnabled = YES"),
                evaluatedWith: app.buttons[AccessibilityIdentifier.purchaseButton.rawValue]
            ),
        ], timeout: 10)
        snapshot("Account")

        // Hit "Log out" button
        _ = app.buttons[AccessibilityIdentifier.logoutButton.rawValue].waitForExistence(timeout: 2)
        app.buttons[AccessibilityIdentifier.logoutButton.rawValue].tap()
        app.alerts.buttons.allElementsBoundByIndex.last?.tap()

        // Wait for Login view to appear after log out
        _ = app.textFields[AccessibilityIdentifier.loginTextField.rawValue].waitForExistence(timeout: 10)
    }
}
