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
    func getAccountWithTime() async -> String {
        if let configuredAccountWithTime = bundleHasTimeAccountNumber, !configuredAccountWithTime.isEmpty {
            return configuredAccountWithTime
        } else {
            let partnerAPIClient = PartnerAPIClient()
            let accountNumber = await partnerAPIClient.createAccount()
            _ = await partnerAPIClient.addTime(accountNumber: accountNumber, days: 1)
            return accountNumber
        }
    }

    /// Delete temporary account with time if a temporary account was used
    func deleteTemporaryAccountWithTime(accountNumber: String) async {
        if bundleHasTimeAccountNumber?.isEmpty == true {
            await PartnerAPIClient().deleteAccount(accountNumber: accountNumber)
        }
    }

    /// Create temporary account without time. Will be created using partner API if token is configured, else falling back to app API
    func createTemporaryAccountWithoutTime() async -> String {
        if let partnerApiToken, !partnerApiToken.isEmpty {
            let partnerAPIClient = PartnerAPIClient()
            return await partnerAPIClient.createAccount()
        } else {
            return mullvadAPIWrapper.createAccount()
        }
    }

    /// Get an account number without time. If an account without time  is specified in the configuration file that account will be used, else a temporary account will be created.
    func getAccountWithoutTime() async -> String {
        if let configuredAccountWithoutTime = bundleNoTimeAccountNumber, !configuredAccountWithoutTime.isEmpty {
            return configuredAccountWithoutTime
        } else {
            let partnerAPIClient = PartnerAPIClient()
            let accountNumber = await partnerAPIClient.createAccount()
            return accountNumber
        }
    }

    /// Delete temporary account withoiut time if a temporary account was used
    func deleteTemporaryAccountWithoutTime(accountNumber: String) async {
        if bundleNoTimeAccountNumber?.isEmpty == true {
            await PartnerAPIClient().deleteAccount(accountNumber: accountNumber)
        }
    }

    func addTime(accountNumber: String, days: Int) async {
        let partnerAPIClient = PartnerAPIClient()
        _ = await partnerAPIClient.addTime(accountNumber: accountNumber, days: days)
    }

    /// Start packet capture for this test case
    @discardableResult
    func performPacketCapture(closure: (PacketCaptureSession) throws -> Void) rethrows -> [Stream] {
        let client = PacketCaptureClient()
        let session = client.startCapture()

        try XCTContext.runActivity(named: "Packet Capture") { activity in
            var captureError: Error?
            do {
                try closure(session)
            } catch {
                captureError = error
            }

            client.stopCapture(session: session)
            let pcap = client.getPCAP(session: session)
            let parsed = client.getParsedCapture(session: session)

            let pcapAttachment = XCTAttachment(data: pcap)
            pcapAttachment.name = self.name + ".pcap"
            pcapAttachment.lifetime = .keepAlways
            activity.add(pcapAttachment)

            let jsonAttachment = XCTAttachment(data: parsed)
            jsonAttachment.name = self.name + ".json"
            jsonAttachment.lifetime = .keepAlways
            activity.add(jsonAttachment)

            if let captureError = captureError {
                throw captureError
            }
        }

        return client.getParsedCaptureObjects(session: session)
    }

    // MARK: - Setup & teardown

    /// Controls which secure settings are reset before launching the app in a UI test.
    class var settingsResetPolicy: UITestSettingsResetPolicy { .none }

    /// Controls the authentication state the app should start with in a UI test.
    class var authenticationState: LaunchArguments.AuthenticationState { .none }

    /// Controls which UserDefaults preferences are reset before launching the app in a UI test.
    class var appPreferencesPolicy: UITestAppPreferencesPolicy { .none }

    class var executableTarget: MullvadExecutableTarget { .uiTests }

    /// Test level setup
    override func setUp() async throws {
        continueAfterFailure = false
        let argumentsJsonString = try? LaunchArguments(
            target: Self.executableTarget,
            areAnimationsDisabled: true,
            authenticationState: Self.authenticationState,
            settingsResetPolicy: Self.settingsResetPolicy,
            appPreferencesResetPolicy: Self.appPreferencesPolicy
        ).toJSON()
        app.launchEnvironment[LaunchArguments.tag] = argumentsJsonString
        app.launch()

        // Wait until the app finishes launching and displays the initial screen.
        agreeToTermsOfServiceIfShown()
        handleRevokedDeviceIfShown()
    }
}

extension XCUIApplication {

    func relaunch(
        _ target: MullvadExecutableTarget = .uiTests,
        authenticationState: LaunchArguments.AuthenticationState = .keepLoggedIn,
        settingsResetPolicy: UITestSettingsResetPolicy = .none,
        file: StaticString = #filePath,
        line: UInt = #line
    ) throws {

        self.terminate()
        XCTAssertTrue(
            self.wait(for: .notRunning, timeout: 5),
            "App did not terminate correctly",
            file: file,
            line: line
        )

        let arguments = LaunchArguments(
            target: target,
            authenticationState: authenticationState,
            settingsResetPolicy: settingsResetPolicy
        )

        self.launchEnvironment[LaunchArguments.tag] = try arguments.toJSON()
        self.launch()
    }
}
