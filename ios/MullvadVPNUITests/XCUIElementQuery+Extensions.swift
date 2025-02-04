//
//  XCUIElementQuery+Extensions.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

extension XCUIElementQuery {
    subscript(key: any RawRepresentable<String>) -> XCUIElement {
        self[key.rawValue]
    }

    subscript(key: AccessibilityIdentifier) -> XCUIElement {
        self[key.asString]
    }
}
