//
//  RelayTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class RelayTests: BaseUITestCase {
    func testAdBlockingViaRelay() throws {
        let app = XCUIApplication()
        app.launch()

        TermsOfServicePage(app)
            .tapAgreeButton()

        Alert(app)
            .tapOkay()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifySuccessIconShown()
            .verifyDeviceLabelShown()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: "Sweden")
            .tapLocationCellExpandButton(withName: "Gothenburg")
            .tapLocationCell(withName: "se-got-wg-001")

        allowAddVPNConfigurations() // Allow adding VPN configurations iOS permission

        TunnelControlPage(app) // Make sure we're taken back to tunnel control page again

        verifyCanReachAdServingDomain()

        TunnelControlPage(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()
            .tapDNSSettingsCell()
            .tapDNSContentBlockingHeaderExpandButton()
            .tapBlockAdsSwitch()

        verifyCannotReachAdServingDomain()
    }

    /// Verify that  ad serving domain is reachable by making sure the host can be found when sending HTTP request to it
    func verifyCanReachAdServingDomain() {
        XCTAssertTrue(canReachAdServingDomain())
    }

    /// Verify that an ad serving domain is NOT reachable by making sure the host cannot be found when sending HTTP request to it
    func verifyCannotReachAdServingDomain() {
        XCTAssertFalse(canReachAdServingDomain())
    }

    /// Attempt to reach HTTP server on an ad serving domain
    /// - Returns: `true` if host can be resolved, otherwise `false`
    private func canReachAdServingDomain() -> Bool {
        guard let url = URL(string: "http://\(adServingDomain)") else { return false }

        let semaphore = DispatchSemaphore(value: 0)
        var requestError: Error?

        let task = URLSession.shared.dataTask(with: url) { _, _, error in
            requestError = error
            semaphore.signal()
        }

        task.resume()

        _ = semaphore.wait(timeout: .distantFuture)

        if let urlError = requestError as? URLError {
            if (urlError.code == .cannotFindHost) {
                return false
            }
        }

        return true
    }
}
