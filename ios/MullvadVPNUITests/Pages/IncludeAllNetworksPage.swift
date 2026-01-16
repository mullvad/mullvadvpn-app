//
//  IncludeAllNetworksPage.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

class IncludeAllNetworksPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.includeAllNetworksView]
        waitForPageToBeShown()
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func tapEnableIncludeAllNetworks() -> Self {
        app.switches[.includeAllNetworksSwitch].tap()
        return self
    }

    @discardableResult func tapEnableLocalNetworkSharing() -> Self {
        app.switches[.localNetworkSharingSwitch].tap()
        return self
    }

    @discardableResult func tapEnableConsent() -> Self {
        app.switches[.actionBox].tap()
        return self
    }

    @discardableResult func verifyConsentIsDisabled() -> Self {
        XCTAssertFalse(app.switches[.actionBox].isEnabled)
        return self
    }

    @discardableResult func tapDismissAlert(failOnUnmetCondition: Bool = false) -> Self {
        app.buttons[.includeAllNetworksNotificationsAlertDismissButton].tap()
        return self
    }

    @discardableResult func verifyFourPages() -> Self {
        XCTAssertEqual(app.pageIndicators.firstMatch.value as? String, "page 1 of 4")
        return self
    }

    @discardableResult func goToLastPage() -> Self {
        let containerView = app.scrollViews[.settingsInfoView]
        containerView.swipeLeft()
        containerView.swipeLeft()
        containerView.swipeLeft()

        print(app.debugDescription)
        return self
    }

    @discardableResult func verifyIncludeAllNetworksSwichIsDisabled() -> Self {
        XCTAssertFalse(app.switches[.includeAllNetworksSwitch].isEnabled)
        return self
    }

    @discardableResult func verifyLocalNetworkSharingSwichIsDisabled() -> Self {
        XCTAssertFalse(app.switches[.localNetworkSharingSwitch].isEnabled)
        return self
    }

    @discardableResult func verifyIncludeAllNetworksSwichIsEnabled() -> Self {
        XCTAssertTrue(app.switches[.includeAllNetworksSwitch].isEnabled)
        return self
    }

    @discardableResult func verifyLocalNetworkSharingSwichIsEnabled() -> Self {
        XCTAssertTrue(app.switches[.localNetworkSharingSwitch].isEnabled)
        return self
    }

    @discardableResult func verifyIncludeAllNetworksSwitchOn() -> Self {
        let switchElement = app.switches[.includeAllNetworksSwitch]

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return self
        }

        XCTAssertEqual(switchValue, "1")
        return self
    }

    @discardableResult func verifyIncludeLocalNetworkSharingSwitchOn() -> Self {
        let switchElement = app.switches[.localNetworkSharingSwitch]

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return self
        }

        XCTAssertEqual(switchValue, "1")
        return self
    }
}
