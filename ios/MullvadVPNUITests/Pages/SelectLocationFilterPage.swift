//
//  SelectLocationFilterPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
        app.cells[AccessibilityIdentifier.ownershipAnyCell].tap()
        return self
    }

    @discardableResult func tapMullvadOwnershipCell() -> Self {
        app.cells[AccessibilityIdentifier.ownershipMullvadOwnedCell].tap()
        return self
    }

    @discardableResult func tapRentedOwnershipCell() -> Self {
        app.cells[AccessibilityIdentifier.ownershipRentedCell].tap()
        return self
    }

    @discardableResult func tapApplyButton() -> Self {
        app.buttons[AccessibilityIdentifier.applyButton].tap()
        return self
    }
}
