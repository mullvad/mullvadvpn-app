//
//  TimeInterval+Timeout.swift
//  MullvadMockData
//
//  Created by Jon Petersson on 2024-06-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

extension TimeInterval {
    struct UnitTest {
        static let timeout: TimeInterval = 60
        static let invertedTimeout: TimeInterval = 0.5
    }
}
