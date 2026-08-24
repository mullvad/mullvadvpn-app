// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public struct NewAccountData: Decodable, Sendable {
    public let id: String
    public let expiry: Date
    public let maxPorts: Int
    public let canAddPorts: Bool
    public let maxDevices: Int
    public let canAddDevices: Bool
    public let number: String

    public init(
        id: String,
        expiry: Date,
        maxPorts: Int,
        canAddPorts: Bool,
        maxDevices: Int,
        canAddDevices: Bool,
        number: String
    ) {
        self.id = id
        self.expiry = expiry
        self.maxPorts = maxPorts
        self.canAddPorts = canAddPorts
        self.maxDevices = maxDevices
        self.canAddDevices = canAddDevices
        self.number = number
    }
}
