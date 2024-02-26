//
//  BaseTestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class BaseUITestCase: XCTestCase {
    let app = XCUIApplication()
    static let defaultTimeout = 5.0

    // swiftlint:disable force_cast
    let displayName = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["DisplayName"] as! String
    let noTimeAccountNumber = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["NoTimeAccountNumber"] as! String
    let hasTimeAccountNumber = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["HasTimeAccountNumber"] as! String
    let fiveWireGuardKeysAccountNumber = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["FiveWireGuardKeysAccountNumber"] as! String
    let iOSDevicePinCode = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["IOSDevicePinCode"] as! String
    // swiftlint:enable force_cast

    /// Handle iOS add VPN configuration permission alert - allow and enter device PIN code
    func allowAddVPNConfigurations() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        let alertAllowButton = springboard.buttons.element(boundBy: 0)
        if alertAllowButton.waitForExistence(timeout: Self.defaultTimeout) {
            alertAllowButton.tap()
        }

        if iOSDevicePinCode.isEmpty == false {
            _ = springboard.buttons["1"].waitForExistence(timeout: Self.defaultTimeout)
            springboard.typeText(iOSDevicePinCode)
        }
    }

    // MARK: - Setup & teardown

    /// Suite level teardown ran after test have executed
    override class func tearDown() {
        uninstallApp()
    }

    /// Test level setup
    override func setUp() {
        continueAfterFailure = false
        app.launch()
    }

    /// Test level teardown
    override func tearDown() {
        app.terminate()
    }

    /// Check if currently logged on to an account. Note that it is assumed that we are logged in if login view isn't currently shown.
    func isLoggedIn() -> Bool {
        return !app
            .otherElements[AccessibilityIdentifier.loginView.rawValue]
            .waitForExistence(timeout: 1.0)
    }

    func agreeToTermsOfServiceIfShown() {
        let termsOfServiceIsShown = app.otherElements[
            AccessibilityIdentifier
                .termsOfServiceView.rawValue
        ]
        .waitForExistence(timeout: 1)

        if termsOfServiceIsShown {
            TermsOfServicePage(app)
                .tapAgreeButton()

            Alert(app) // Changes alert
                .tapOkay()

            LoginPage(app)
        }
    }

    /// Login with specified account number. It is a prerequisite that the login page is currently shown.
    func login(accountNumber: String) {
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(accountNumber)
            .tapAccountNumberSubmitButton()
            .verifyDeviceLabelShown()
    }

    func logoutIfLoggedIn() {
        if isLoggedIn() {
            HeaderBar(app)
                .tapAccountButton()

            AccountPage(app)
                .tapLogOutButton()

            LoginPage(app)
        }
    }

    static func uninstallApp() {
        let appName = "Mullvad VPN"

        let timeout = TimeInterval(5)
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        let spotlight = XCUIApplication(bundleIdentifier: "com.apple.Spotlight")

        springboard.swipeDown()
        spotlight.textFields["SpotlightSearchField"].typeText(appName)

        let appIcon = spotlight.icons[appName].firstMatch
        if appIcon.waitForExistence(timeout: timeout) {
            appIcon.press(forDuration: 2)
        } else {
            XCTFail("Failed to find app icon named \(appName)")
        }

        let deleteAppButton = spotlight.buttons["Delete App"]
        if deleteAppButton.waitForExistence(timeout: timeout) {
            deleteAppButton.tap()
        } else {
            XCTFail("Failed to find 'Delete App'")
        }

        let finalDeleteButton = springboard.alerts.buttons["Delete"]
        if finalDeleteButton.waitForExistence(timeout: timeout) {
            finalDeleteButton.tap()
        } else {
            XCTFail("Failed to find 'Delete'")
        }
    }
}
