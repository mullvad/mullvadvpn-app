//
//  DaitaConfiguration.swift
//  MullvadTypes
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

/// Contains arguments needed to initialize DAITA for a WireGuard device.
public struct DaitaConfiguration: Equatable, Sendable {
    /// Contains a string describing a set of DAITA machines.
    public let machines: String
    /// Maximum amount of DAITA events to enqueue at any given time.
    public let maxEvents: UInt32
    /// Maximum amount of DAITA actions to enqueue at any given time.
    public let maxActions: UInt32
    /// Maximum amount of DAITA padding to enqueue at any given time.
    public let maxPadding: Double
    /// Maximum amount of DAITA blocking to enqueue at any given time.
    public let maxBlocking: Double

    public init(machines: String, maxEvents: UInt32, maxActions: UInt32, maxPadding: Double, maxBlocking: Double) {
        self.machines = machines
        self.maxEvents = maxEvents
        self.maxActions = maxActions
        self.maxPadding = maxPadding
        self.maxBlocking = maxBlocking
    }
}
