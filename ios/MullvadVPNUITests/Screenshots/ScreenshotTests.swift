//
//  ScreenshotTests.swift
//  MullvadVPNScreenshots
//
//  Created by Jon Petersson on 2024-05-28.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class ScreenshotTests: BaseUITestCase {
    override func setUp() {
        setupSnapshot(app, waitForAnimations: false)

        let argumentsJsonString = try? LaunchArguments(
            target: .screenshots,
            areAnimationsDisabled: true
        ).toJSON()
        app.launchEnvironment[LaunchArguments.tag] = argumentsJsonString

        super.setUp()

        agreeToTermsOfServiceIfShown()
        dismissChangeLogIfShown()

        if !isLoggedIn() {
            login(accountNumber: hasTimeAccountNumber)
        }

        // Reset connection state.
        if TunnelControlPage(app).connectionIsSecured {
            TunnelControlPage(app)
                .tapDisconnectButton()
        }
    }

    func testTakeScreenshotOfQuantumSecuredConnection() throws {
        // We can't close banners in the screenshot tests due to how the NotificationController view
        // is overridden, so we need to restart the app once to make sure the "new device" notification
        // isn't visible.
        app.terminate()
        app.launch()

        // Reset stored state.
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapVPNSettingsCell()

            VPNSettingsPage(self.app)
                .tapQuantumResistantTunnelExpandButton()
                .tapQuantumResistantTunnelOffCell()
        }

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapQuantumResistantTunnelExpandButton()
            .tapQuantumResistantTunnelOnCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: "Sweden")

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        snapshot("QuantumConnectionSecured")
    }

    func testTakeScreenshotOfCustomListSelected() throws {
        let customListName = "Low latency locations"

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .tapCustomListEllipsisButton()
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

    func testTakeScreenshotOfRelayFilter() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapFilterButton()

        SelectLocationFilterPage(app)
            .tapOwnershipCellExpandButton()
            .tapProvidersCellExpandButton()

        snapshot("RelayFilter")
    }

    func testTakeScreenshotOfVPNSettings() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        snapshot("VPNSettings")
    }

    func testTakeScreenshotOfDNSSettings() throws {
        // Reset stored state.
        addTeardownBlock {
            DNSSettingsPage(self.app)
                .tapBlockAdsSwitch()
                .tapBlockTrackerSwitch()
                .tapBlockMalwareSwitch()
                .tapBlockAdultContentSwitch()
                .tapBlockGamblingSwitch()
                .tapBlockSocialMediaSwitch()
        }

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

    func testTakeScreenshotOfAccount() throws {
        HeaderBar(app)
            .tapAccountButton()

        snapshot("Account")
    }
}
