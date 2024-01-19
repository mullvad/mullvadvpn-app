//
//  RelayTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class RelayTests: LoggedInWithTimeUITestCase {
    func testAdBlockingViaDNS() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()
            .tapDNSSettingsCell()
            .tapDNSContentBlockingHeaderExpandButton()
            .tapBlockAdsSwitch()
            .swipeDownToDismissModal()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurations() // Allow adding VPN configurations iOS permission

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        Networking.verifyCannotReachAdServingDomain()
    }
}
