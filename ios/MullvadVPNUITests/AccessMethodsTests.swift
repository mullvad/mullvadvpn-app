//
//  AccessMethodsTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-09-30.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccessMethodsTests: LoggedOutUITestCase {
    func testEncryptedDNS() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        let cell = APIAccessPage(app)
            .getAccessMethodCell(accessibilityId: AccessibilityIdentifier.accessMethodEncryptedDNSCell)

        XCTAssertFalse(APIAccessPage(app).getMethodIsDisabled(cell))

        cell.tap()

        EditAccessMethodPage(app)
            .tapEnableMethodSwitch()
            .tapBackButton()

        XCTAssertTrue(APIAccessPage(app).getMethodIsDisabled(cell))
    }
}
