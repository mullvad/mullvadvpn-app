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
        app.switches[.includeAllNetworksSwitch].switches.firstMatch.tap()
        return self
    }

    @discardableResult func tapEnableLocalNetworkSharing() -> Self {
        app.switches[.localNetworkSharingSwitch].switches.firstMatch.tap()
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
        app.buttons[.includeAllNetworksNotificationsAlertDismissButton]
            .tapWhenHittable(failOnUnmetCondition: failOnUnmetCondition)
        return self
    }

    @discardableResult func verifyFourPages() -> Self {
        XCTAssertEqual(app.pageIndicators.firstMatch.value as? String, "page 1 of 4")
        return self
    }

    @discardableResult func goToLastPage() -> Self {
        let pages = pageContainer()

        guard let position = pageIndicatorPosition() else { return self }

        for _ in position.page..<position.pageCount {
            pages.swipeLeft()
        }

        let lastPosition = pageIndicatorPosition()
        XCTAssertEqual(lastPosition?.page, lastPosition?.pageCount, "Failed to swipe to the last page")

        return self
    }

    /// The element to swipe to page through the info pages.
    ///
    /// From iOS 27 the pages are a collection view nested in the scroll view, and swiping the scroll view
    /// itself does not page it.
    private func pageContainer() -> XCUIElement {
        let scrollView = app.scrollViews[.settingsInfoView]
        let collectionView = scrollView.collectionViews.firstMatch

        return collectionView.exists ? collectionView : scrollView
    }

    /// Reads the page indicator - used to determine when to stop scrolling
    private func pageIndicatorPosition() -> (page: Int, pageCount: Int)? {
        let numbers = (app.pageIndicators.firstMatch.value as? String)?
            .components(separatedBy: " ")
            .compactMap { Int($0) }

        guard let numbers, numbers.count == 2 else {
            XCTFail("Failed to read page indicator")
            return nil
        }

        return (numbers[0], numbers[1])
    }

    @discardableResult func verifyIncludeAllNetworksSwitchIsDisabled() -> Self {
        XCTAssertFalse(app.switches[.includeAllNetworksSwitch].switches.firstMatch.isEnabled)
        return self
    }

    @discardableResult func verifyLocalNetworkSharingSwitchIsDisabled() -> Self {
        XCTAssertFalse(app.switches[.localNetworkSharingSwitch].switches.firstMatch.isEnabled)
        return self
    }

    @discardableResult func verifyIncludeAllNetworksSwitchIsEnabled() -> Self {
        XCTAssertTrue(app.switches[.includeAllNetworksSwitch].switches.firstMatch.isEnabled)
        return self
    }

    @discardableResult func verifyLocalNetworkSharingSwitchIsEnabled() -> Self {
        XCTAssertTrue(app.switches[.localNetworkSharingSwitch].switches.firstMatch.isEnabled)
        return self
    }

    @discardableResult func verifyIncludeAllNetworksSwitchOn() -> Self {
        let switchElement = app.switches[.includeAllNetworksSwitch].switches.firstMatch

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return self
        }

        XCTAssertEqual(switchValue, "1")
        return self
    }

    @discardableResult func verifyIncludeLocalNetworkSharingSwitchOn() -> Self {
        let switchElement = app.switches[.localNetworkSharingSwitch].switches.firstMatch

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return self
        }

        XCTAssertEqual(switchValue, "1")
        return self
    }
}
