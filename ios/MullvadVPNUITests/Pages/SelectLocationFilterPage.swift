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

class SelectLocationFilterPage: Page {
    override init(_ app: XCUIApplication) {
        super.init(app)
    }

    @discardableResult func tapOwnershipCellExpandButton() -> Self {
        app.otherElements[AccessibilityIdentifier.locationFilterOwnershipHeaderCell]
            .buttons[AccessibilityIdentifier.expandButton].tap()
        return self
    }

    @discardableResult func tapProvidersCellExpandButton() -> Self {
        app.otherElements[AccessibilityIdentifier.locationFilterProvidersHeaderCell]
            .buttons[AccessibilityIdentifier.expandButton].tap()
        return self
    }

    @discardableResult func tapAnyOwnershipCell() -> Self {
        app.buttons[AccessibilityIdentifier.ownershipAnyCell].tap()
        return self
    }

    @discardableResult func tapMullvadOwnershipCell() -> Self {
        app.buttons[AccessibilityIdentifier.ownershipMullvadOwnedCell].tap()
        return self
    }

    @discardableResult func tapRentedOwnershipCell() -> Self {
        app.buttons[AccessibilityIdentifier.ownershipRentedCell].tap()
        return self
    }

    @discardableResult func tapApplyButton() -> Self {
        app.buttons[AccessibilityIdentifier.applyFilterButton].tap()
        return self
    }
}
