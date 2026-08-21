// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

public struct DaitaV2Parameters: Equatable, Sendable {
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
