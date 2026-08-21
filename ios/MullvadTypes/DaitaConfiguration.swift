// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
