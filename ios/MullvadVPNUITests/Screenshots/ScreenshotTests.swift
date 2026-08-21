// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

@MainActor
class ScreenshotTests: LoggedInWithTimeUITestCase {
    override class var executableTarget: MullvadExecutableTarget {
        .screenshots
    }
    override class var settingsResetPolicy: UITestSettingsResetPolicy { .only([.customRelayLists, .settings]) }

    override func setUp() async throws {
        setupSnapshot(app, waitForAnimations: false)
        try await super.setUp()
    }

    func testTakeScreenshotOfQuantumSecuredConnection() async throws {
        // We can't close banners in the screenshot tests due to how the NotificationController view
        // is overridden, so we need to restart the app once to make sure the "new device" notification
        // isn't visible.
        try app.relaunch(Self.executableTarget)

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: "Sweden")

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        snapshot("QuantumConnectionSecured")
    }

    func testTakeScreenshotOfCustomListSelected() async throws {
        let customListName = "Low latency locations"

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .tapAddNewCustomList()

        CustomListPage(app)
            .renameCustomList(name: customListName)
            .addOrEditLocations()

        AddCustomListLocationsPage(app)
            .scrollToLocationWith(identifier: "Sweden")
            .unfoldLocationwith(identifier: "Sweden")
            .unfoldLocationwith(identifier: "Gothenburg")
            .toggleLocationCheckmarkWith(identifier: "se-got-wg-101")
            .scrollToLocationWith(identifier: "Germany")
            .unfoldLocationwith(identifier: "Germany")
            .toggleLocationCheckmarkWith(identifier: "Berlin")
            .scrollToLocationWith(identifier: "Finland")
            .toggleLocationCheckmarkWith(identifier: "Finland")
            .tapBackButton()

        CustomListPage(app)
            .tapCreateListButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: customListName)

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: customListName)

        snapshot("CustomListSelected")
    }

    func testTakeScreenshotOfRelayFilter() async throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapExitFilterButton()

        snapshot("RelayFilter")
    }

    func testTakeScreenshotOfVPNSettings() async throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        snapshot("VPNSettings")
    }

    func testTakeScreenshotOfDNSSettings() async throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapDNSSettingsCell()

        DNSSettingsPage(app)
            .tapDNSContentBlockersHeaderExpandButton()
            .tapBlockAdsSwitch()
            .tapBlockTrackerSwitch()
            .tapBlockMalwareSwitch()
            .tapBlockAdultContentSwitch()
            .tapBlockGamblingSwitch()
            .tapBlockSocialMediaSwitch()

        snapshot("DNSSettings")
    }

    func testTakeScreenshotOfAccount() async throws {
        HeaderBar(app)
            .tapAccountButton()

        snapshot("Account")
    }
}
