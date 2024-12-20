//
//  DaitaV2Parameters.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2024-11-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct DaitaV2Parameters: Equatable {
    public let machines: String
    public let maximumEvents: UInt32
    public let maximumActions: UInt32
    public let maximumPadding: Double
    public let maximumBlocking: Double

    public init(
        machines: String,
        maximumEvents: UInt32,
        maximumActions: UInt32,
        maximumPadding: Double,
        maximumBlocking: Double
    ) {
        self.machines = machines
        self.maximumEvents = maximumEvents
        self.maximumActions = maximumActions
        self.maximumPadding = maximumPadding
        self.maximumBlocking = maximumBlocking
    }
}
