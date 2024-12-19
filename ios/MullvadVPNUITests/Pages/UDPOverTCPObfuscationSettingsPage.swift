//
//  UDPOverTCPObfuscationSettingsPage.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-12-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

    @discardableResult func tapPortCell(_ index: Int) -> Self {
        portCell(index).tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.navigationBars.buttons.element(boundBy: 0).tap()
        return self
    }
}
