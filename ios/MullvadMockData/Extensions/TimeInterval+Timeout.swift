//
//  TimeInterval+Timeout.swift
//  MullvadMockData
//
//  Created by Jon Petersson on 2024-06-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

extension TimeInterval {
    struct UnitTest {
        static let timeout: TimeInterval = 10
        static let invertedTimeout: TimeInterval = 0.5
    }
}
