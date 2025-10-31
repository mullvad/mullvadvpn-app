//
//  ScreenshotTests.swift
//  MullvadVPNScreenshots
//
//  Created by Jon Petersson on 2024-05-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@MainActor
class ScreenshotTests: LoggedInWithTimeUITestCase {
    override func setUp() async throws {
        setupSnapshot(app, waitForAnimations: false)

        let argumentsJsonString = try? LaunchArguments(
            target: .screenshots,
            areAnimationsDisabled: true
        ).toJSON()
        app.launchEnvironment[LaunchArguments.tag] = argumentsJsonString

        try await super.setUp()
    }

    func testTakeScreenshotOfQuantumSecuredConnection() async throws {
        // We can't close banners in the screenshot tests due to how the NotificationController view
        // is overridden, so we need to restart the app once to make sure the "new device" notification
        // isn't visible.
        app.terminate()
        app.launch()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: "Sweden")

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
            .scrollToLocationWith(identifier: "se")
            .unfoldLocationwith(identifier: "se")
            .unfoldLocationwith(identifier: "se-got")
            .toggleLocationCheckmarkWith(identifier: "se-got-wg-101")
            .scrollToLocationWith(identifier: "de")
            .unfoldLocationwith(identifier: "de")
            .toggleLocationCheckmarkWith(identifier: "de-ber")
            .scrollToLocationWith(identifier: "fi")
            .toggleLocationCheckmarkWith(identifier: "fi")
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
            .tapMenuButton()
            .tapFilterButton()

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
