//
//  UDPOverTCPObfuscationSettingsPage.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-12-12.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class UDPOverTCPObfuscationSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    private var table: XCUIElement {
        app.collectionViews[AccessibilityIdentifier.wireGuardObfuscationUdpOverTcpTable]
    }

    private func portCell(_ index: Int) -> XCUIElement {
        table.cells.element(boundBy: index)
    }

    @discardableResult func tapAutomaticPortCell() -> Self {
        portCell(0).tap()
        return self
    }

    @discardableResult func tapPort80Cell() -> Self {
        portCell(1).tap()
        return self
    }

    @discardableResult func tapPort443Cell() -> Self {
        portCell(2).tap()
        return self
    }

    @discardableResult func tapPort5001Cell() -> Self {
        portCell(3).tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.navigationBars.buttons.element(boundBy: 0).tap()
        return self
    }
}
