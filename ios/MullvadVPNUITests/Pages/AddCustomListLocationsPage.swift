//
//  AddCustomListLocationsPage.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2024-06-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AddCustomListLocationsPage: EditCustomListLocationsPage {
    @discardableResult override func tapBackButton() -> Self {
        app.navigationBars["Locations"].buttons.firstMatch.tap()
        return self
    }
}
