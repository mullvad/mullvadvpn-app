//
//  SelectLocationPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class SelectLocationPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.selectLocationView]
        waitForPageToBeShown()
    }

    @discardableResult func tapEntryLocationButton() -> Self {
        app.buttons[AccessibilityIdentifier.entryLocationButton]
            .tap()
        return self
    }

    @discardableResult func tapLocationCell(withName name: String) -> Self {
        let cell = app.buttons[AccessibilityIdentifier.locationListItem(name)]
        app.scrollDownToElement(element: cell)
        cell.tap()
        return self
    }

    @discardableResult func tapLocationCellExpandButton(withName name: String) -> Self {
        let cell = app.buttons[AccessibilityIdentifier.locationListItem(name)]
        app.scrollDownToElement(element: cell)
        cell.wait(for: .hittable)

        // The expand chevron is a fixed-width button at the trailing edge of the
        // row. Because .accessibilityElement(children: .combine) merges children
        // into a single element, the chevron cannot be queried individually.
        // Calculate the tap position dynamically from the cell's actual width so
        // it works on any device size.
        let chevronCenter = 1.0 - (28.0 / cell.frame.width)

        // Retry the tap if the cell didn't expand — the first tap after a scroll
        // can be absorbed by scroll deceleration.
        for _ in 0..<3 {
            cell.coordinate(withNormalizedOffset: CGVector(dx: chevronCenter, dy: 0.5)).tap()

            // Poll for the accessibility value to update after the expand animation.
            let deadline = Date().addingTimeInterval(2)
            while Date() < deadline {
                if cell.value as? String == "Expanded" {
                    return self
                }
                RunLoop.current.run(until: Date().addingTimeInterval(0.2))
            }
        }

        XCTFail("Failed to expand location cell '\(name)' after multiple attempts")
        return self
    }

    @discardableResult func tapAddNewCustomList() -> Self {
        let addNewCustomListButton = app.buttons[AccessibilityIdentifier.addNewCustomListButton]
        addNewCustomListButton.tap()
        return self
    }

    @discardableResult func editExistingCustomLists() -> Self {
        let editCustomListsButton = app.buttons[AccessibilityIdentifier.editCustomListButton]
        editCustomListsButton.tap()
        return self
    }

    @discardableResult func cellWithIdentifier(identifier: AccessibilityIdentifier) -> XCUIElement {
        app.buttons[identifier]
    }

    @discardableResult func tapFilterButton() -> Self {
        app.buttons[AccessibilityIdentifier.selectLocationFilterButton]
            .firstMatch
            .tap()
        return self
    }

    @discardableResult func tapMenuButton() -> Self {
        app.images[AccessibilityIdentifier.selectLocationToolbarMenu].tap()
        return self
    }

    @discardableResult func tapDoneButton() -> Self {
        app.buttons[AccessibilityIdentifier.closeSelectLocationButton].tap()
        return self
    }

    @discardableResult func tapToggleMultihop() -> Self {
        app.buttons[AccessibilityIdentifier.toggleMultihopButton].tap()
        return self
    }

    @discardableResult func tapToggleRecents() -> Self {
        app.buttons[AccessibilityIdentifier.recentConnectionsToggleButton].tap()
        return self
    }

    @discardableResult func verifyRecentIsDisabled() -> Self {
        let textElement = app.buttons["Enable recents"]
        XCTAssertTrue(textElement.existsAfterWait())
        return self
    }

    func locationCellIsExpanded(_ name: String) -> Bool {
        let cell = app.buttons[AccessibilityIdentifier.locationListItem(name)]
        guard cell.exists else { return false }
        return cell.value as? String == "Expanded"
    }

    func verifyEditCustomListsButtonIs(enabled: Bool) {
        let editCustomListsButton = app.buttons[AccessibilityIdentifier.editCustomListButton]
        XCTAssertTrue(editCustomListsButton.isEnabled == enabled)
    }

    @discardableResult func verifyMultihopOff() -> Self {
        let textElement = app.buttons["Enable multihop"]

        XCTAssertTrue(textElement.exists)

        return self
    }

    @discardableResult func verifyMultihopOn() -> Self {
        let textElement = app.buttons["Disable multihop"]

        XCTAssertTrue(textElement.exists)

        return self
    }

    @discardableResult func enableRecents() -> Self {
        let recentButton = app.buttons[AccessibilityIdentifier.recentConnectionsToggleButton]
        if recentButton.label.contains("Enable") {
            recentButton.tap()
        }
        return self
    }

    @discardableResult func disableRecents() -> Self {
        let recentButton = app.buttons[AccessibilityIdentifier.recentConnectionsToggleButton]
        if recentButton.label.contains("Disable") {
            recentButton.tap()
            DisableRecentsConfirmationAlert(app)
                .tapDisableRecentConnectionsButton()
        }
        return self
    }

}

/// Confirmation alert displayed when disabling recents
private class DisableRecentsConfirmationAlert: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.otherElements[.alertContainerView]
        waitForPageToBeShown()
    }

    @discardableResult func tapDisableRecentConnectionsButton() -> Self {
        app.buttons[AccessibilityIdentifier.disableRecentConnectionsButton].tap()
        return self
    }
}
