//
//  DAITASettingsTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-09-27.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import XCTest

final class DAITASettingsTests: XCTestCase {
    func testShouldDoDirectOnly() throws {
        let settings = DAITASettings()

        XCTAssertEqual(
            settings.shouldDoAutomaticRouting,
            settings.daitaState == .on && settings.directOnlyState == .off
        )
    }
}
