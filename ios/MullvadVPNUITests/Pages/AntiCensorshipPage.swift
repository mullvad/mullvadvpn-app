//
//  AntiCensorshipPage.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2026-08-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class AntiCensorshipPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)
    }

    @discardableResult func tapWireGuardObfuscationAutomaticCell() -> Self {
        app.buttons[.wireGuardObfuscationAutomatic].tap()
        return self
    }

    @discardableResult func selectObfuscationUdpOverTcp() -> Self {
        app.buttons[.wireGuardObfuscationUdpOverTcp].tap()
        return self
    }

    @discardableResult func navigateToUDPOverTCPObfuscationSettings() -> Self {
        app.buttons[.wireGuardObfuscationUdpOverTcpPort].tap()
        return self
    }

    @discardableResult func selectObfuscationShadowsocks() -> Self {
        app.buttons[.wireGuardObfuscationShadowsocks].tap()
        return self
    }

    @discardableResult func navigateToShadowsocksObfuscationSettings() -> Self {
        app.buttons[.wireGuardObfuscationShadowsocksPort].tap()
        return self
    }

    @discardableResult func selectObfuscationQuic() -> Self {
        app.buttons[.wireGuardObfuscationQuic].tap()
        return self
    }

    @discardableResult func selectObfuscationLwo() -> Self {
        app.buttons[.wireGuardObfuscationLwo].tap()
        return self
    }

    @discardableResult func navigateToLwoObfuscationSettings() -> Self {
        app.buttons[.wireGuardObfuscationLwoPort].tap()
        return self
    }

    @discardableResult func tapWireGuardObfuscationOffCell() -> Self {
        app.buttons[.wireGuardObfuscationOff].tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.navigationBars.buttons.element(boundBy: 0).tap()
        return self
    }
}
