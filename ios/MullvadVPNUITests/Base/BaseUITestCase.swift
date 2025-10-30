//
//  BaseTestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-12.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

@MainActor
class BaseUITestCase: XCTestCase {
    let app = XCUIApplication()
    static let defaultTimeout = 5.0
    static let longTimeout = 15.0
    static let veryLongTimeout = 20.0
    static let extremelyLongTimeout = 180.0
    static let shortTimeout = 1.0

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

    static func uninstallAppInTestSuiteTearDown() -> Bool {
        if let uninstallAppInTestSuiteTearDown = Bundle(for: BaseUITestCase.self)
            .infoDictionary?["UninstallAppInTestSuiteTearDown"] as? String
        {
            return uninstallAppInTestSuiteTearDown == "1"
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

    /// Handle iOS add VPN configuration permission alert if presented, otherwise ignore
    func allowAddVPNConfigurationsIfAsked() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        if springboard.buttons["Allow"].waitForExistence(timeout: Self.shortTimeout) {
            let alertAllowButton = springboard.buttons["Allow"]
            if alertAllowButton.waitForExistence(timeout: Self.defaultTimeout) {
                alertAllowButton.tap()
            }

            if iOSDevicePinCode.isEmpty == false {
                _ = springboard.buttons["1"].waitForExistence(timeout: Self.defaultTimeout)
                springboard.typeText(iOSDevicePinCode)
            }
        }
    }

    /// Handle iOS local network access permission alert if presented, otherwise ignore
    func allowLocalNetworkAccessIfAsked() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        if springboard.buttons["Allow"].waitForExistence(timeout: Self.shortTimeout) {
            let alertAllowButton = springboard.buttons["Allow"]
            if alertAllowButton.waitForExistence(timeout: Self.defaultTimeout) {
                alertAllowButton.tap()
            }
        }
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

    /// Override this class function to change the uninstall behaviour in suite level teardown
    class func shouldUninstallAppInTeardown() -> Bool {
        return true
    }

    /// Suite level teardown ran after all tests in suite have been executed
    override class func tearDown() {
        // This function is not marked `@MainActor` therefore cannot legally enter its context without help
        Task { @MainActor in
            if shouldUninstallAppInTeardown() && uninstallAppInTestSuiteTearDown() {
                uninstallApp()
            }
        }
    }

    /// Test level setup
    override func setUp() async throws {
        currentTestCaseShouldCapturePackets = false  // Reset for each test case run
        continueAfterFailure = false
        app.launch()
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
            .waitForExistence(timeout: 1.0)
    }

    func isPresentingSettings() -> Bool {
        return
            app
            .otherElements[.settingsContainerView]
            .exists
    }

    func agreeToTermsOfServiceIfShown() {
        let termsOfServiceIsShown = app.otherElements[
            .termsOfServiceView
        ]
        .waitForExistence(timeout: Self.shortTimeout)

        if termsOfServiceIsShown {
            TermsOfServicePage(app)
                .tapAgreeButton()
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

        repeat {
            successIconShown = LoginPage(app)
                .tapAccountNumberSubmitButton()
                .getSuccessIconShown()

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

        HeaderBar(app)
            .verifyDeviceLabelShown()
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
                    .waitForLogoutSpinnerToDisappear()
            } else {
                // Workaround for revoked device view not showing account button
                RevokedDevicePage(app)
                    .tapGoToLogin()
            }

            LoginPage(app)
        }
    }

    static func uninstallApp() {
        let appName = "Mullvad VPN"
        let searchQuery =
            appName
            .replacingOccurrences(
                of: " ",
                with: ""
            )  // With space in the query Spotlight search sometimes don't match the Mullvad VPN app

        let timeout = TimeInterval(5)
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        let spotlight = XCUIApplication(bundleIdentifier: "com.apple.Spotlight")

        /// iPhone uses spotlight, iPad uses springboard. But the usage is quite similar
        let spotlightOrSpringboard = BaseUITestCase.testDeviceIsIPad() ? springboard : spotlight
        var mullvadAppIcon: XCUIElement

        // How to navigate to Spotlight search differs between iPhone and iPad
        if BaseUITestCase.testDeviceIsIPad() == false {  // iPhone
            springboard.swipeDown()
            spotlight.textFields["SpotlightSearchField"].typeText(searchQuery)
            mullvadAppIcon = spotlightOrSpringboard.icons[appName]
        } else {  // iPad
            // Swipe left enough times to reach the last page
            for _ in 0..<3 {
                springboard.swipeLeft()
                Thread.sleep(forTimeInterval: 0.5)
            }

            springboard.swipeDown()
            springboard.searchFields.firstMatch.typeText(searchQuery)
            mullvadAppIcon = spotlightOrSpringboard.icons.matching(identifier: appName).allElementsBoundByIndex[1]
        }

        // The rest of the delete app flow is same for iPhone and iPad with the exception that iPhone uses spotlight and iPad uses springboard
        if mullvadAppIcon.waitForExistence(timeout: timeout) {
            mullvadAppIcon.press(forDuration: 2)
        } else {
            XCTFail("Failed to find app icon named \(appName)")
        }

        let deleteAppButton = spotlightOrSpringboard.buttons["Delete App"]
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
