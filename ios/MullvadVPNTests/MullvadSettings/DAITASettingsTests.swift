//
//  DAITASettingsTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-09-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import XCTest

final class DAITASettingsTests: XCTestCase {
    func testIsAutomaticRouting() throws {
        let settings = DAITASettings()

        XCTAssertEqual(
            settings.isAutomaticRouting,
            settings.daitaState == .on && settings.directOnlyState == .off
        )
    }

    func testIsDirectOnly() throws {
        let settings = DAITASettings()

        XCTAssertEqual(
            settings.isDirectOnly,
            settings.daitaState == .on && settings.directOnlyState == .on
        )
    }
}
