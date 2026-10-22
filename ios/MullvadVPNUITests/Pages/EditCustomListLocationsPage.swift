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

class EditCustomListLocationsPage: Page {
    enum Action {
        case add, edit
    }

    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.scrollViews[.editCustomListEditLocationsView]
        waitForPageToBeShown()
    }

    @discardableResult func toggleLocationCheckmarkWith(identifier: String, scrollUpwards: Bool = false) -> Self {
        let button = app.switches[.customListLocationCheckmarkButton(identifier)]
        if scrollUpwards {
            app.scrollUpToElement(element: button)
        } else {
            app.scrollDownToElement(element: button)
        }
        button.tap()
        return self
    }

    @discardableResult func tapLocationCellExpandButton(withName name: String, scrollUpwards: Bool = false) -> Self {
        let button = app.switches[AccessibilityIdentifier.customListLocationCheckmarkButton(name)]
        if scrollUpwards {
            app.scrollUpToElement(element: button)
        } else {
            app.scrollDownToElement(element: button)
        }
        button.wait(for: .hittable)

        // The expand chevron is a fixed-width button at the trailing edge of the
        // row. Because .accessibilityElement(children: .combine) merges children
        // into a single element, the chevron cannot be queried individually.
        // Calculate the tap position dynamically from the cell's actual width so
        // it works on any device size.
        let chevronCenter = 1.0 - (28.0 / button.frame.width)

        // Retry the tap if the cell didn't expand — the first tap after a scroll
        // can be absorbed by scroll deceleration.
        for _ in 0..<3 {
            button.coordinate(withNormalizedOffset: CGVector(dx: chevronCenter, dy: 0.5)).tap()

            // Poll for the accessibility value to update after the expand animation.
            let deadline = Date().addingTimeInterval(2)
            while Date() < deadline {
                if button.value as? String == "Expanded" {
                    return self
                }
                RunLoop.current.run(until: Date().addingTimeInterval(0.2))
            }
        }

        XCTFail("Failed to expand location cell '\(name)' after multiple attempts")
        return self
    }

    @discardableResult func tapBackButton(title: String) -> Self {
        app.navigationBars[title].buttons.firstMatch.tap()
        return self
    }
}
