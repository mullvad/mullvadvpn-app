//
//  AddCustomListLocationsPage.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2024-06-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AddCustomListLocationsPage: EditCustomListLocationsPage {
    @discardableResult override func tapBackButton() -> Self {
        app.navigationBars["Add locations"].buttons.firstMatch.tap()
        return self
    }
}
