// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

extension BaseUITestCase {
    func disableAccessMethods(_ methods: [AccessibilityIdentifier]) {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        for methodIdentifier in methods {
            APIAccessPage(app)
                .getAccessMethodCell(accessibilityId: methodIdentifier)
                .tap()

            EditAccessMethodPage(app)
                .tapEnableMethodSwitch()
                .tapBackButton()
        }

        // Navigate back to main screen
        let backButton = app.navigationBars.firstMatch.buttons.firstMatch
        backButton.tap()

        SettingsPage(app)
            .tapDoneButton()
    }

    /// make the internet unreachable, by adding a broken access method and disabling everything else
    func breakConnectivity() {
        let customAccessMethodName = "Disable-access-dummy"
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .tapAddButton()

        allowLocalNetworkAccessIfAsked()

        AddAccessMethodPage(app)
            .tapNameCell()
            .enterText(customAccessMethodName)
            .tapTypeCell()
            .tapSOCKS5TypeValueCell()
            .tapServerCell()
            .enterText("123.123.123.123")
            .dismissKeyboard()
            .tapPortCell()
            .enterText("123")
            .dismissKeyboard()
            .tapAddButton()
            .waitForAPIUnreachableLabel()

        AddAccessMethodAPIUnreachableAlert(app)
            .tapSaveButton()

        var accessMethods = APIAccessPage(app).getAccessMethodCells()
        accessMethods.removeLast()
        for cell in accessMethods {
            cell.tap()
            EditAccessMethodPage(app)
                .tapEnableMethodSwitch()
                .tapBackButton()
        }

        SettingsPage(app)
            .swipeDownToDismissModal()

    }

    /// Check if currently logged on to an account. Note that it is assumed that we are logged in if login view isn't currently shown.
    func loginViewIsShown() -> Bool {
        return
            app
            .otherElements[.loginView]
            .existsAfterWait()
    }

    func isPresentingSettings() -> Bool {
        return
            app
            .otherElements[.settingsContainerView]
            .exists
    }

    func agreeToTermsOfServiceIfShown() {
        let timeout: XCUIElement.Timeout =
            Self.authenticationState == .forceLoggedOut && loginViewIsShown() ? .short : .longerThanMullvadAPITimeout

        if app.otherElements[.termsOfServiceView].existsAfterWait(timeout: timeout) {
            TermsOfServicePage(app)
                .tapAgreeButton()
        }
    }

    func handleRevokedDeviceIfShown() {
        if app.otherElements[.revokedDeviceView].existsAfterWait(timeout: .short) {
            RevokedDevicePage(app)
                .tapGoToLogin()
        }
    }

    /// Handle iOS local network access permission alert if presented, otherwise ignore
    func allowLocalNetworkAccessIfAsked() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        springboard.buttons["Allow"].tapWhenHittable(timeout: .short, failOnUnmetCondition: false)
    }

    /// Handle iOS add VPN configuration permission alert if presented, otherwise ignore
    func allowAddVPNConfigurationsIfAsked() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        let alertAllowButton = springboard.buttons["Allow"]
        if alertAllowButton.existsAfterWait(timeout: .short) {
            alertAllowButton.tapWhenHittable()

            if !iOSDevicePinCode.isEmpty {

                // Springboard sometimes has digit buttons, sometimes they are keys?
                let passcodeScreenVisible =
                    springboard.buttons["1"].existsAfterWait()
                    || springboard.keys["1"].existsAfterWait()
                    || springboard.secureTextFields.firstMatch.existsAfterWait()
                if passcodeScreenVisible {
                    springboard.typeText(iOSDevicePinCode)
                }
            }
        }
    }

    /// Login with specified account number. It is a prerequisite that the login page is currently shown.
    func login(accountNumber: String) {
        var successIconShown = false
        var retryCount = 0
        let maxRetryCount = 3

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(accountNumber)
            .tapAccountNumberSubmitButton()

        repeat {
            successIconShown = LoginPage(app).getSuccessIconShown()

            if successIconShown == false {
                // If the login happened too fast, the UI harness will miss the success icon being shown
                // Check if the app is already on main page, and continue if it is.
                if app.otherElements[.headerBarView].exists {
                    successIconShown = true
                    break
                }

                // Give it some time to show up. App might be waiting for a network connection to timeout.
                LoginPage(app).waitForAccountNumberSubmitButton()
            }

            retryCount += 1
        } while successIconShown == false && retryCount < maxRetryCount
        removeExistingDeviceSessionIfNeeded()

        skipNotificationPromptIfShown()

        HeaderBar(app)
            .verifyDeviceLabelShown()
    }

    private func skipNotificationPromptIfShown() {
        if app.otherElements[.notificationPromptView].existsAfterWait(timeout: .short) {
            NotificationPromptPage(app)
                .tapSkipButton()
        }
    }

    private func removeExistingDeviceSessionIfNeeded() {
        if app.otherElements[.deviceManagementView].existsAfterWait(timeout: .short) {
            DeviceManagementPage(app)
                .waitForDeviceList()
                .tapRemoveDeviceButton(cellIndex: 1)

            DeviceManagementLogOutDeviceConfirmationAlert(app)
                .tapYesLogOutDeviceButton()

            DeviceManagementPage(app)
                .waitForDeviceList()
                .waitForNoLoading()
                .tapContinueWithLoginButton()
        }
    }

    func logoutIfLoggedIn() {
        if loginViewIsShown() {
            // First dismiss settings modal if presented
            if isPresentingSettings() {
                SettingsPage(app)
                    .swipeDownToDismissModal()
            }

            if app.buttons[AccessibilityIdentifier.accountButton].exists {
                HeaderBar(app)
                    .tapAccountButton()
                AccountPage(app)
                    .tapLogOutButton()
            } else {
                // Workaround for revoked device view not showing account button
                RevokedDevicePage(app)
                    .tapGoToLogin()
            }

            LoginPage(app)
        }
    }
}
