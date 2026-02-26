//
//  BaseTestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

@MainActor
class BaseUITestCase: XCTestCase {
    let app = XCUIApplication()

    /// The apps default country - the preselected country location after fresh install
    static let appDefaultCountry = "Sweden"

    /// Default country to use in tests
    static let testsDefaultCountryName = "Sweden"
    static let testsDefaultCountryIdentifier = "se"

    /// Default DAITA supported relay to use in tests
    static let testsDefaultDAITACountryName = "Germany"
    static let testsNonDAITACountryName = "Ireland"

    /// Default city to use in tests
    static let testsDefaultCityName = "Gothenburg"
    static let testsDefaultCityIdentifier = "se-got"

    /// Default Mullvad owned relays to use in tests
    static let testsDefaultMullvadOwnedCityName = "Stockholm"
    static let testsDefaultMullvadOwnedRelayName = "se-sto-wg-001"

    /// Default relay to use in tests
    static let testsDefaultRelayName = "se-got-wg-001"

    /// Default QUIC supported relay to use in tests
    static let testsDefaultQuicCountryName = "Germany"
    static let testsDefaultQuicCityName = "Frankfurt"
    static let testsDefaultQuicRelayName = "de-fra-wg-001"

    /// True when the current test case is capturing packets
    private var currentTestCaseShouldCapturePackets = false

    /// True when a packet capture session is active
    private var packetCaptureSessionIsActive = false
    private var packetCaptureSession: PacketCaptureSession?

    let displayName =
        Bundle(for: BaseUITestCase.self)
        .infoDictionary?["DisplayName"] as! String
    private let bundleHasTimeAccountNumber =
        Bundle(for: BaseUITestCase.self)
        .infoDictionary?["HasTimeAccountNumber"] as? String
    private let bundleNoTimeAccountNumber =
        Bundle(for: BaseUITestCase.self)
        .infoDictionary?["NoTimeAccountNumber"] as? String
    let iOSDevicePinCode =
        Bundle(for: BaseUITestCase.self)
        .infoDictionary?["IOSDevicePinCode"] as! String
    let attachAppLogsOnFailure =
        Bundle(for: BaseUITestCase.self)
        .infoDictionary?["AttachAppLogsOnFailure"] as! String == "1"
    let partnerApiToken = Bundle(for: BaseUITestCase.self).infoDictionary?["PartnerApiToken"] as? String

    lazy var mullvadAPIWrapper: MullvadAPIWrapper = {
        do {
            return try! MullvadAPIWrapper()
        }
    }()

    static func testDeviceIsIPad() -> Bool {
        if let testDeviceIsIPad = Bundle(for: BaseUITestCase.self).infoDictionary?["TestDeviceIsIPad"] as? String {
            return testDeviceIsIPad == "1"
        }

        return false
    }

    /// Get an account number with time. If an account with time is specified in the configuration file that account will be used, else a temporary account will be created if partner API token has been configured.
    func getAccountWithTime() -> String {
        if let configuredAccountWithTime = bundleHasTimeAccountNumber, !configuredAccountWithTime.isEmpty {
            return configuredAccountWithTime
        } else {
            let partnerAPIClient = PartnerAPIClient()
            let accountNumber = partnerAPIClient.createAccount()
            _ = partnerAPIClient.addTime(accountNumber: accountNumber, days: 1)
            return accountNumber
        }
    }

    /// Delete temporary account with time if a temporary account was used
    func deleteTemporaryAccountWithTime(accountNumber: String) {
        if bundleHasTimeAccountNumber?.isEmpty == true {
            PartnerAPIClient().deleteAccount(accountNumber: accountNumber)
        }
    }

    /// Create temporary account without time. Will be created using partner API if token is configured, else falling back to app API
    func createTemporaryAccountWithoutTime() -> String {
        if let partnerApiToken, !partnerApiToken.isEmpty {
            let partnerAPIClient = PartnerAPIClient()
            return partnerAPIClient.createAccount()
        } else {
            return mullvadAPIWrapper.createAccount()
        }
    }

    /// Get an account number without time. If an account without time  is specified in the configuration file that account will be used, else a temporary account will be created.
    func getAccountWithoutTime() -> String {
        if let configuredAccountWithoutTime = bundleNoTimeAccountNumber, !configuredAccountWithoutTime.isEmpty {
            return configuredAccountWithoutTime
        } else {
            let partnerAPIClient = PartnerAPIClient()
            let accountNumber = partnerAPIClient.createAccount()
            return accountNumber
        }
    }

    /// Delete temporary account withoiut time if a temporary account was used
    func deleteTemporaryAccountWithoutTime(accountNumber: String) {
        if bundleNoTimeAccountNumber?.isEmpty == true {
            PartnerAPIClient().deleteAccount(accountNumber: accountNumber)
        }
    }

    /// Handle iOS add VPN configuration permission alert if presented, otherwise ignore
    func allowAddVPNConfigurationsIfAsked() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        let alertAllowButton = springboard.buttons["Allow"]
        if alertAllowButton.existsAfterWait(timeout: .short) {
            alertAllowButton.tap()
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

    /// Handle iOS local network access permission alert if presented, otherwise ignore
    func allowLocalNetworkAccessIfAsked() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        springboard.buttons["Allow"].tapWhenHittable(timeout: .short, failOnUnmetCondition: false)
    }

    /// Start packet capture for this test case
    func startPacketCapture() {
        currentTestCaseShouldCapturePackets = true
        packetCaptureSessionIsActive = true
        let packetCaptureClient = PacketCaptureClient()
        packetCaptureSession = packetCaptureClient.startCapture()
    }

    /// Stop the current packet capture and return captured traffic
    func stopPacketCapture() -> [Stream] {
        packetCaptureSessionIsActive = false
        guard let packetCaptureSession else {
            XCTFail("Trying to stop capture when there is no active capture")
            return []
        }

        let packetCaptureAPIClient = PacketCaptureClient()
        packetCaptureAPIClient.stopCapture(session: packetCaptureSession)
        let capturedData = packetCaptureAPIClient.getParsedCaptureObjects(session: packetCaptureSession)

        return capturedData
    }

    // MARK: - Setup & teardown

    /// Override this class function to control whether the app state
    /// should be reset during the test suite setup phase.
    ///
    /// Return `all` to ensure the application starts from a clean state
    /// before tests are executed.
    class var settingsResetPolicy: LaunchArguments.LocalDataResetPolicy {
        return .all
    }

    class var authenticationState: LaunchArguments.AuthenticationState {
        return .forceLoggedOut
    }

    /// Test level setup
    override func setUp() async throws {
        currentTestCaseShouldCapturePackets = false  // Reset for each test case run
        continueAfterFailure = false

        let argumentsJsonString = try? LaunchArguments(
            target: .uiTests,
            authenticationState: Self.authenticationState,
            localDataResetPolicy: Self.settingsResetPolicy,
        ).toJSON()
        app.launchEnvironment[LaunchArguments.tag] = argumentsJsonString
        app.launch()

        // Wait until the app finishes launching and displays the initial screen.
        agreeToTermsOfServiceIfShown()
    }

    func agreeToTermsOfServiceIfShown() {
        let timeout: XCUIElement.Timeout =
            Self.authenticationState == .forceLoggedOut ? .longerThanMullvadAPITimeout : .short
        if app.otherElements["termsOfServiceView"].existsAfterWait(timeout: timeout) {
            TermsOfServicePage(app)
                .tapAgreeButton()
        }
    }

    /// Test level teardown
    override func tearDown() async throws {
        if currentTestCaseShouldCapturePackets {
            guard let packetCaptureSession = packetCaptureSession else {
                XCTFail("Packet capture session unexpectedly not set up")
                return
            }

            let packetCaptureClient = PacketCaptureClient()

            // If there's a an active session due to cancelled/failed test run make sure to end it
            if packetCaptureSessionIsActive {
                packetCaptureSessionIsActive = false
                packetCaptureClient.stopCapture(session: packetCaptureSession)
            }

            let pcapFileContents = packetCaptureClient.getPCAP(session: packetCaptureSession)
            let parsedCapture = packetCaptureClient.getParsedCapture(session: packetCaptureSession)
            self.packetCaptureSession = nil

            let pcapAttachment = XCTAttachment(data: pcapFileContents)
            pcapAttachment.name = self.name + ".pcap"
            pcapAttachment.lifetime = .keepAlways
            self.add(pcapAttachment)

            let jsonAttachment = XCTAttachment(data: parsedCapture)
            jsonAttachment.name = self.name + ".json"
            jsonAttachment.lifetime = .keepAlways
            self.add(jsonAttachment)
        }

        app.terminate()

        if let testRun = self.testRun, testRun.failureCount > 0, attachAppLogsOnFailure == true {
            app.launch()

            HeaderBar(app)
                .tapSettingsButton()

            SettingsPage(app)
                .tapReportAProblemCell()

            ProblemReportPage(app)
                .tapViewAppLogsButton()

            let logText = AppLogsPage(app)
                .getAppLogText()

            // Attach app log to result
            let dateFormatter = DateFormatter()
            dateFormatter.dateFormat = "yyyy-MM-dd_HH-mm-ss"
            let dateString = dateFormatter.string(from: Date())
            let attachment = XCTAttachment(string: logText)
            attachment.name = "app-log-\(dateString).log"
            add(attachment)

            app.terminate()
        }
    }

    /// Check if currently logged on to an account. Note that it is assumed that we are logged in if login view isn't currently shown.
    func isLoggedIn() -> Bool {
        return
            !app
            .otherElements[.loginView]
            .existsAfterWait(timeout: .short)
    }

    func isPresentingSettings() -> Bool {
        return
            app
            .otherElements[.settingsContainerView]
            .exists
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
        if isLoggedIn() {
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

    func relaunch() throws {
        app.terminate()
        XCTAssertTrue(app.wait(for: .notRunning, timeout: 5))

        let arguments = LaunchArguments(
            target: .uiTests,
            authenticationState: .keepLoggedIn,
            localDataResetPolicy: .none
        )

        app.launchEnvironment[LaunchArguments.tag] = try arguments.toJSON()
        app.launch()
    }
}
